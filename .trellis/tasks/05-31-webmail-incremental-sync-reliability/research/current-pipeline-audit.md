# 当前 Webmail 邮件同步链路审计

## 已确认链路

Provider worker 按账号运行：Gmail/IMAP/Outlook worker 写入 SQLite 后通过 `message_tx` 交给 `server/src/rpc/indexing.rs::index_new_messages`。该管线负责规则处理、浏览器通知、搜索索引以及 SSE 事件。前端由 `src/components/StatusBar.tsx` 监听 SSE 并失效 React Query 缓存。

## 关键发现

1. `mail:new` 当前在规则处理前发送，前端可能看到规则前状态。
2. 初始全量同步会对每封入库邮件发送 `mail:new`，可导致 shell/messages/threads 反复重拉。
3. `wake_sync(ensureRunning=true)` 先启动 worker，再发送 trigger；刚启动 worker 本身已有 startup pass，可能紧接着多跑一轮 poll。
4. SSE 使用 broadcast channel，无事件序号和 replay；断线期间事件会丢失。
5. Gmail History API 当前没有显式 status 检查，也没有处理分页，长期可靠性有风险。
6. 旧账号级 `/api/accounts/:id/sync/trigger` 可绕过 `/api/sync/wake` 的 passive trigger 契约。

## 本任务改造原则

- 周期 poll completion 只代表心跳，不驱动前端数据刷新。
- 只有“数据实际变化”事件才刷新列表缓存。
- 初始/批量导入必须聚合刷新，避免每封邮件引发一次网络重拉。
- SSE 断线恢复必须有兜底 catch-up。
- Gmail cursor 只在完整、成功处理所有分页后推进。
