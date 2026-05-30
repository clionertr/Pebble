# Webmail 启动快照与增量更新

## Goal

把 Webmail 前端从“多个组件各自请求 accounts / folders / unreadCounts / gmail-realtime / inbox”的模式，演进为“启动快照 + React Query 缓存复用 + 同步唤醒 + 后续增量修正”的数据管道。目标是减少首屏和设置页的接口风暴，同时保持邮件列表、文件夹、未读数和 Gmail realtime 配置在网络断开、SSE 重连、后台同步后仍能及时、可靠地更新。

## What I already know

* 用户认可的数据模型：`/api/shell` 负责首次快照，`/api/sync/wake` 负责唤醒同步，后续通过 SSE 与增量补丁更新前端缓存。
* 已完成并提交的相关基础能力：`POST /api/sync/wake`，提交 `b1ae7c0 feat: add batched sync wake endpoint`。
* 当前后端已有 `GET /api/shell`，返回 `accounts`、按账号分组的 `folders`、按账号分组的 `unreadCounts`。
* 当前 `getFolderUnreadCounts(accountId)` 已经通过 `client.getShell()` 读取 unread counts，但 `useAccountsQuery` / `useFoldersForAccountsQuery` 仍会分别请求 `/api/accounts` 和每个账号的 `/api/accounts/:id/folders`。
* 当前 Gmail realtime 配置在 `AccountsTab` 中会对每个 Gmail 账号调用 `getGmailRealtimeConfig(accountId)`；账号编辑子表单也会单账号再次读取。
* 当前收件箱列表仍使用分页：`useMessagesQuery` 每页 50 条，`useThreadsQuery` 每次 50 条；这符合“不全量拉邮件正文/全量列表”的方向。
* 当前实时刷新方式主要在 `StatusBar.refreshMailQueries()` 中收到 `mail:new`、`mail:sync-progress completed`、`mail:sync-complete`、`mail:pending-ops-changed` 后 `invalidateQueries`，不是增量 patch。
* 当前 SSE 客户端有自动重连，但事件本身没有全局 sequence/cursor；如果浏览器断网或 SSE 断开期间漏事件，只能靠后续 invalidate/refetch 补。

## Assumptions (temporary)

* MVP 优先扩展和消费 `/api/shell`，先解决启动/设置页多接口请求；真正的 `/api/mail/changes?since=...` 明确放到后续任务。
* “第一次拉取全量”指全量账号元数据、文件夹、未读数、Gmail realtime 配置，以及当前视图首屏邮件/线程；不指拉取所有邮件或所有正文。
* Gmail realtime 配置属于低频元数据，适合放进 shell 快照并 hydrate 到 React Query cache。
* 邮件及时性短期仍依赖现有后端同步 worker + SSE + React Query invalidate；增量 patch 需要新增后端 change log 或足够可靠的事件序列。
* 网络断开恢复时，前端应先 `wakeSync({ reason: "network_online", ensureRunning: true })`，再通过 shell/current inbox refetch 或 changes 补齐。

## Decisions

* 本任务先做 Shell 快照整合 MVP：扩展 `/api/shell`，前端 hydrate React Query 缓存，网络恢复时 `wakeSync + shell/current inbox refetch`；真正的 `/api/mail/changes?since=seq` / change log / 增量 patch 放到后续任务。
* 严格测试先行：先写能约束接口契约、缓存 hydrate、Gmail realtime 去 N+1 请求、网络恢复 refetch 的失败测试，再实现代码。

## Open Questions

* 暂无阻塞问题。

## Requirements (evolving)

* 扩展 `/api/shell`，使其成为 Webmail 启动快照的权威入口。
* 前端新增或改造 shell query/hydration，让账号、文件夹、未读数、Gmail realtime 配置优先来自 shell 缓存。
* 保持 inbox/messages/threads 分页读取，不拉取全量邮件正文。
* 保持 `mail:new`、sync progress、pending ops 事件能刷新当前视图数据；不能牺牲及时性。
* 网络 offline 时不发无意义请求；恢复 online 后唤醒同步并补齐缓存。
* 设置页 Gmail realtime 配置不应再对所有 Gmail 账号 N 次请求，除非用户打开单账号编辑且缓存缺失。
* 保留底层单资源接口作为 fallback 和局部刷新能力。

## Acceptance Criteria (evolving)

* [x] 首次进入应用后，accounts/folders/unreadCounts/gmailRealtime 元数据可由一次 shell 请求填充 React Query 缓存。
* [x] 多账号侧边栏和 InboxView 不再因为账号数量线性增加 folders 请求。
* [x] AccountsTab 展示 Gmail realtime 状态时优先使用 shell 缓存，不再为每个 Gmail 账号并发请求配置。
* [x] 邮件列表仍按当前文件夹/聚合文件夹分页拉取首屏和后续页。
* [x] 网络从 offline 恢复 online 时，调用 sync wake，并补齐 shell/当前邮件列表缓存。
* [x] SSE `mail:new` 后，当前列表/线程/文件夹/未读数不会长期陈旧。
* [x] 周期性 `mail:sync-progress(status="completed", phase="poll")` 不再触发 shell/messages/threads 重拉。
* [x] 前端 API 序列化、hook 行为、缓存 hydrate 有测试覆盖。
* [x] Rust API/shell 响应契约有测试或现有 API 测试覆盖。
* [x] 测试必须先于实现编写，并能在实现前暴露当前缺口。

## Definition of Done (team quality bar)

* Tests added/updated (unit/integration where appropriate)
* Lint / typecheck / CI green
* Docs/notes updated if behavior changes
* Rollout/rollback considered if risky

## Out of Scope (explicit)

* 不把所有账号合并成一个后端同步 worker。
* 不在首屏拉取所有历史邮件或所有邮件正文。
* 不移除现有 `/api/accounts/:id/folders`、`/api/accounts/:id/gmail-realtime` 等底层接口。
* 不改变服务商侧 Gmail history / Outlook delta / IMAP IDLE 的同步机制。

## Technical Notes

* Relevant backend files:
  * `server/src/api/shell.rs`：已有 shell 快照，只含 accounts/folders/unreadCounts。
  * `server/src/api/accounts.rs`：folders、gmail-realtime 单账号接口。
  * `server/src/api/messages.rs`：`GET /api/inbox` 分页入口。
  * `server/src/rpc/sync_cmd.rs`：已新增 batched wake。
* Relevant frontend files:
  * `src/lib/api-client.ts` / `src/lib/api.ts`：HTTP client 与领域 API 包装。
  * `src/hooks/queries/useAccountsQuery.ts`、`useFoldersQuery.ts`、`useFolderUnreadCounts.ts`：当前元数据查询入口。
  * `src/features/settings/AccountsTab.tsx`：Gmail realtime 多账号配置请求。
  * `src/features/inbox/InboxView.tsx` / `ThreadView.tsx`：当前分页列表读取。
  * `src/components/StatusBar.tsx`：SSE 事件后刷新缓存。
  * `src/lib/sse-client.ts`：SSE 重连，但无 sequence/cursor。
* Relevant specs:
  * `.trellis/spec/pebble/backend/webmail-api-contracts.md`：Webmail REST/SSE 契约、SSE 缓存刷新、sync wake 契约。
  * `.trellis/spec/guides/cross-layer-thinking-guide.md`：本任务横跨 API/service/frontend cache/SSE。
  * `.trellis/spec/guides/code-reuse-thinking-guide.md`：避免继续散落多个元数据请求入口。

## Research References

* 暂无外部研究；本阶段主要基于现有代码结构与项目规范。若进入真正 change log / delta patch 设计，需要补充本地研究文档。
