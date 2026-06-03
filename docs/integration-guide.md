# Pebble Webmail 集成速查

本文面向需要接入、反向代理或调试 Pebble Webmail 的开发者/运维者。所有示例假设前端与后端同源部署。

## 鉴权模型

Pebble 是单用户 Webmail。登录后服务端设置 `pebble_session` Cookie：

- `HttpOnly`
- `Secure`
- `SameSite=Strict`
- 7 天有效期

除登录、状态检查、OAuth 和 Gmail webhook 等少量公开入口外，`/api/*` 和 `/events` 都要求有效 Cookie。

常用入口：

| 方法 | 路径 | 说明 |
|---|---|---|
| `POST` | `/api/auth/login` | 用密码登录 |
| `GET` | `/api/auth/status` | 检查当前会话 |
| `POST` | `/api/auth/logout` | 登出 |
| `GET` | `/events` | SSE 实时事件流 |
| `GET` | `/api/docs/openapi.json` | OpenAPI JSON |

## 启动快照 `/api/shell`

首屏账号元数据应优先通过一次 shell 请求获取：

```http
GET /api/shell
```

响应结构：

```json
{
  "accounts": [],
  "folders": {
    "account-id": []
  },
  "unreadCounts": {
    "account-id": {
      "folder-id": 3
    }
  },
  "gmailRealtime": {
    "gmail-account-id": {
      "accountId": "gmail-account-id",
      "enabled": true,
      "status": "active",
      "configMissing": false,
      "topicName": "projects/.../topics/...",
      "expirationMs": 1770000000000,
      "lastWatchHistoryId": "12345",
      "lastWatchAt": 1760000000000,
      "lastError": null,
      "fallbackIntervalMinutes": 15
    }
  }
}
```

约定：

- 字段必须存在；没有数据时返回空数组或空对象。
- `gmailRealtime` 只包含 Gmail 账号，非 Gmail 账号不返回伪配置。
- 文件夹或 Gmail realtime 局部失败时，shell 仍尽量返回其它账号数据。
- 单资源接口仍可用于局部操作，例如 `/api/accounts/:id/folders` 和 `/api/accounts/:id/gmail-realtime`。

## 邮件列表读取

邮件列表是分页查询，不通过 shell 全量返回：

```http
GET /api/inbox?folderId=<folder-id>&limit=50&offset=0
GET /api/inbox?folderId=<first-folder>&limit=50&folderIds=<id1>,<id2>,<id3>
```

多账号聚合视图会把多个 inbox 文件夹合并成一次查询，所以 URL 中的 `folderIds` 可能较长。这是正常的合并分页查询。

线程列表：

```http
GET /api/threads?folderId=<folder-id>&limit=50&offset=0
```

## 同步唤醒 `/api/sync/wake`

前端被动事件和手动同步统一调用：

```http
POST /api/sync/wake
Content-Type: application/json

{
  "account_ids": ["account-1", "account-2"],
  "reason": "network_online",
  "ensure_running": true,
  "poll_interval_secs": 3
}
```

字段说明：

| 字段 | 说明 |
|---|---|
| `account_ids` | 可省略；省略表示所有账号，空数组表示不唤醒任何账号 |
| `reason` | `manual`、`window_focus`、`window_blur`、`network_online`、`startup`、`provider_push` 等 |
| `ensure_running` | 是否先确保账号 worker 已启动 |
| `poll_interval_secs` | `ensure_running=true` 时的 worker 轮询间隔 |

## SSE 事件与缓存刷新

`GET /events` 使用同一个 session Cookie。重要事件：

| 事件 | 含义 | 前端建议 |
|---|---|---|
| `mail:new` | 新邮件已入库并可展示 | 刷新 shell、messages、threads、对应账号 folders/unreadCounts |
| `mail:pending-ops-changed` | 待写回远端操作变化 | 刷新 pending ops 与当前邮件视图 |
| `mail:sync-progress` | 同步 pass 状态变化 | 更新状态；`status=completed, phase=poll` 不应重拉列表 |
| `mail:sync-complete` | one-shot worker 退出兼容事件 | 刷新相关缓存 |
| `mail:realtime-status` | 后端实时模式/错误状态 | 更新状态栏和设置页展示 |

不要把常规 `poll` 完成当作数据变化，否则低延迟轮询会退化成每几秒重拉 `/api/shell` 和 `/api/inbox`。

## 反向代理要求

公网反代必须代理这些路径到后端：

```nginx
location ~ ^/(api|events|auth|webhook) {
    proxy_pass http://127.0.0.1:3000;
    proxy_set_header Host $host;
    proxy_set_header X-Real-IP $remote_addr;

    # SSE 必须关闭缓冲
    proxy_buffering off;
    proxy_cache off;
    proxy_read_timeout 3600s;
}
```

前端静态资源由 nginx 或容器中的前端服务托管；API/SSE/OAuth/webhook 走后端。

## SSE 事件全集

`GET /events` 通过同源 Cookie 鉴权。每个事件是 `event: <name>` + `data: <json>`。心跳每 15 秒一次。

| 事件 | 载荷 | 触发时机 |
|---|---|---|
| `mail:new` | `{ account_id, message_id, folder_ids, thread_id, subject, from, received_at, latency: { source, backend_received_at_ms, backend_sse_at_ms, message_received_at_ms, history_id } }` | 新邮件完成索引（含规则引擎）后 |
| `mail:sync-progress` | `{ account_id, ... }`（结构按 provider 略有差异） | Gmail/IMAP/Outlook 同步过程进度 |
| `mail:sync-complete` | `{ account_id, ... }` | 完整同步 pass 结束 |
| `mail:realtime-status` | `{ account_id, mode, provider, last_success_at, next_retry_at, message }`（`mode` ∈ `realtime`/`polling`/`manual`/`backoff`/`offline`/`auth_required`/`error`） | Gmail Watch 续约、同步操作后的实时模式状态 |
| `mail:error` | `pebble_mail::SyncError` | 同步过程中出现的错误 |
| `mail:unsnoozed` | `{ message_id, return_to }` | 暂延消息到点自动恢复或手动取消暂延 |
| `mail:pending-ops-changed` | `{}` | 待处理操作队列变化（新增/取消/删除/处理完成） |
| `mail:attachment-download-progress` | `{ attachment_id, bytes_copied, total_bytes }` | 附件流式下载进度 |

注意：不要把 `mail:sync-progress` 的常规 `poll` 完成当作数据变化，否则前端会频繁重拉 `/api/shell` 和 `/api/inbox`。

## 推送通知（Web Push / VAPID）

浏览器订阅流程：

1. `GET /api/notifications/vapid-public-key` → `{ "public_key": "<URL-safe base64>" }`
2. 浏览器调用 `PushManager.subscribe({ userVisibleOnly: true, applicationServerKey: publicKey })` 获得 `PushSubscription`。
3. `POST /api/notifications/subscriptions`：

   ```json
   {
     "device_id": "string",
     "device_name": "string | null",
     "subscription": {
       "endpoint": "https://push.example.com/...",
       "keys": { "p256dh": "...", "auth": "..." }
     }
   }
   ```

   首次订阅时后端会自动推送一封未读摘要。

4. 管理：`GET /api/notifications/devices` 列表，`PATCH /api/notifications/devices/:device_id` 改名，`DELETE /api/notifications/devices/:device_id` 删除，`POST /api/notifications/test` 发一条测试推送（`{ "device_id": "..." }`）。

VAPID 密钥默认自动生成并持久化到加密存储。多实例部署需设置 `PEBBLE_VAPID_PRIVATE_KEY`（可选 `PEBBLE_VAPID_PUBLIC_KEY` 校验）保证一致性。

服务端推送载荷形状：

```json
{
  "kind": "mail" | "otp" | "mail_batch" | "summary" | "test",
  "title": "string",
  "body": "string",
  "url": "string",
  "tag": "string",
  "timestamp": 0,
  "allow_foreground": false,
  "message_id": "string | null"
}
```

## Kanban

| 方法 | 路径 | 请求 | 响应 |
|---|---|---|---|
| `GET` | `/api/kanban` | `?column=todo\|waiting\|done`（可选） | `{ "cards": [{ message_id, column, position, created_at, updated_at }], "notes": { "messageId": "note" } }` |
| `POST` | `/api/kanban/cards` | `{ "messageId": "string", "column": "todo\|waiting\|done", "position": 0 }` | `null` |
| `DELETE` | `/api/kanban/cards/:messageId` | — | `null` |
| `GET` | `/api/kanban/notes` | — | `{ "messageId": "note" }` |
| `PUT` | `/api/kanban/notes/:messageId` | `{ "note": "string" }` | 全量 notes map |
| `PATCH` | `/api/kanban/notes` | `{ "notes": { "messageId": "note" } }` 或直接传 notes map | 合并后的 notes map（不覆盖已有） |

## 暂延（Snooze）

| 方法 | 路径 | 请求 | 响应 |
|---|---|---|---|
| `GET` | `/api/snoozed` | — | `[{ message_id, snoozed_at, unsnoozed_at, return_to }]` |
| `POST` | `/api/snoozed` | `{ "messageId": "string", "until": 0, "returnTo": "string" }`（`until` 为 Unix 时间戳） | `null` |
| `DELETE` | `/api/snoozed/:messageId` | — | `null`；同时发 `mail:unsnoozed` SSE |

## 待处理操作（Pending Ops）

所有写回远端失败或异步队列化的操作在此管理。

| 方法 | 路径 | 请求 | 响应 |
|---|---|---|---|
| `GET` | `/api/pending-ops` | `?accountId=...&limit=100`（默认 100，上限 500） | `[{ id, account_id, message_id, op_type, status, attempts, last_error, created_at, updated_at, next_retry_at }]` |
| `GET` | `/api/pending-ops/summary` | `?accountId=...` | `{ pending_count, in_progress_count, failed_count, total_active_count, last_error, updated_at }` |
| `POST` | `/api/pending-ops/:id/cancel` | — | `null`；发 `mail:pending-ops-changed` |
| `DELETE` | `/api/pending-ops/:id` | — | `null`；发 `mail:pending-ops-changed` |
