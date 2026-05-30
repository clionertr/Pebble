# Pebble Webmail 架构说明

本文面向接手 Pebble Webmail 的开发者，说明浏览器、后端 API、同步 worker 与前端缓存之间如何协作。

## 总体结构

```text
Browser React SPA
  ├─ HTTP REST: /api/*
  ├─ SSE:       /events
  └─ OAuth:     /auth/login, /auth/callback
        │
        ▼
Axum backend
  ├─ server/src/api/        Web API 边界
  ├─ server/src/rpc/        业务服务层
  ├─ crates/pebble-store    SQLite 数据
  ├─ crates/pebble-mail     Gmail / IMAP / Outlook 同步
  ├─ crates/pebble-search   Tantivy 搜索
  └─ crates/pebble-privacy  HTML 清理与追踪保护
```

所有 `/api/*` 与 `/events` 请求都使用 `pebble_session` Cookie 鉴权。前端生产代码只走 HTTP/SSE，不再使用 Tauri IPC。

## 启动快照与前端缓存

Webmail 首屏的账号元数据由 `GET /api/shell` 一次返回：

- `accounts`：账号列表
- `folders`：按账号分组的文件夹列表
- `unreadCounts`：按账号和文件夹分组的未读数
- `gmailRealtime`：按 Gmail 账号分组的实时推送配置

前端通过 `src/hooks/queries/useShellQuery.ts` 把 shell 快照写入 React Query：

| 数据 | Cache key |
|---|---|
| Shell 原始快照 | `['shell']` |
| 账号列表 | `['accounts']` |
| 单账号文件夹 | `['folders', accountId]` |
| 单账号未读数 | `['folder-unread-counts', accountId]` |
| Gmail realtime 配置 | `['gmail-realtime', accountId]` |

这样首屏和设置页不会再按账号数量线性请求 `accounts + folders + gmail-realtime`。底层单资源接口仍保留，作为局部刷新和后续兼容入口。

## 邮件列表仍然分页

Shell 快照只包含元数据，不包含所有历史邮件或正文。邮件列表继续分页读取：

- 消息列表：`GET /api/inbox?folderId=...&limit=50&offset=...&folderIds=...`
- 线程列表：`GET /api/threads?folderId=...&limit=50&offset=...&folderIds=...`
- 正文/渲染内容：按用户打开的消息再按需请求

当处于多账号聚合收件箱时，`folderIds` 可能很长；这是一次合并查询，不是每个文件夹单独请求。

## 实时性与刷新边界

及时性主要由三层保证：

1. 后端同步 worker 从 Gmail / IMAP / Outlook 拉取变化。
2. 后端通过 `/events` 发送 `mail:new`、`mail:pending-ops-changed`、`mail:sync-progress`、`mail:sync-complete` 等事件。
3. 前端只在“确有变化”的事件上失效缓存。

前端刷新规则：

| 事件 | 前端动作 |
|---|---|
| `mail:new` | invalidate `['shell']`、`['messages']`、`['threads']`，并按账号刷新 folders/unreadCounts；延迟刷新搜索 |
| `mail:pending-ops-changed` | 刷新 pending ops 摘要，并刷新相关邮件/元数据缓存 |
| 网络 offline → online | 调用 `POST /api/sync/wake`，随后刷新 `['shell']`、`['messages']`、`['threads']` |
| `mail:sync-progress` `status='completed'` `phase='poll'` | 只更新同步状态为 idle，不刷新 shell/inbox |
| `mail:sync-complete` | 兼容 one-shot worker 退出场景，刷新相关缓存 |

关键原则：周期性 poll 完成只是“巡检结束”，不等于数据变化。不能因为它每几秒重拉 `/api/shell` 或 `/api/inbox`。

## 同步唤醒入口

被动事件和手动同步统一通过 `POST /api/sync/wake` 表达意图。前端不应对每个账号循环调用 start/trigger。

常见 reason：

- `window_focus`
- `window_blur`
- `network_online`
- `manual`
- `startup`
- `provider_push`

`ensure_running=true` 用于 focus、network_online、startup 等需要确保 worker 存活的场景；manual-only 模式下的被动事件不应唤醒同步。

## 设计取舍

- 保留账号级 worker：每个账号仍有独立 stop/trigger/backoff/provider 状态，避免一个全局 worker 隔离性太差。
- Shell 只聚合低频元数据：不把所有邮件或正文塞进启动快照，避免首屏变重。
- 事件驱动优先：SSE 和网络恢复负责补齐变化；周期性 poll 完成不触发全量重拉。
- 后续真正增量补丁可在此基础上增加 `changes?since=cursor`，但当前实现以 shell 快照 + 分页列表 + 事件失效为边界。
