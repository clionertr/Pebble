# Webmail 增量同步可靠性改造

## Goal

把当前 Webmail 邮件同步链路从“近似实时 + 局部缓存”提升为更清晰的“首次全量、之后增量、断线可补偿”的事件驱动模型，减少重复请求，同时确保新邮件、规则处理、搜索和前端缓存的状态一致。

## Requirements

1. **`mail:new` 代表规则后的最终状态**
   - 后端必须在规则处理、消息重载、最终 folder_ids 确定后再发送前端可见的邮件变化事件。
   - payload 中的 `folder_ids` 必须是规则处理后的最终 folder IDs。
   - 浏览器 Web Push 仍沿用规则后的通知判断。

2. **前端刷新合并 / 降噪**
   - `mail:new` 多次快速到达时，前端应合并 invalidate，避免初始全量同步或批量变更造成每封邮件一次 `/api/shell`、`/api/inbox`、`/api/threads`。
   - 搜索缓存仍要延迟刷新，避免查询早于 Tantivy commit。
   - 普通 `mail:sync-progress(status="completed", phase="poll")` 不得刷新 shell/messages/threads。

3. **wake 启动去重**
   - `POST /api/sync/wake` 在 `ensure_running=true` 且 worker 是刚启动时，不应立刻再投递一次会导致重复 poll 的 trigger。
   - 已存在 worker 时仍应投递 trigger。
   - passive trigger，如 `window_blur`，不得启动缺失 worker。

4. **SSE 重连补偿**
   - 前端 SSE 重连成功后必须触发一次 catch-up：唤醒同步并失效关键缓存，弥补断线期间丢失的事件。
   - catch-up 应避免首次连接时无意义刷新，只在断线后重连触发。

5. **Gmail History API 可靠性**
   - Gmail History API 必须显式检查 HTTP status。
   - 401 映射为 Auth 错误；404 history expired 必须触发可恢复路径或明确错误，不得静默成功。
   - 必须处理 `nextPageToken` 全部分页。
   - cursor 只能在所有分页和本地处理成功后推进。

6. **旧 trigger 入口契约收紧**
   - `/api/accounts/:id/sync/trigger` 不得让 `window_blur` 等不需要立即同步的 passive trigger 在缺 worker 时启动 one-shot。
   - `manual` 缺 worker 时仍允许 one-shot。

## Acceptance Criteria

- [x] Rust 测试覆盖 `mail:new` payload 使用规则后 folder_ids。
- [x] Rust 测试覆盖 `wake_sync` 刚启动 worker 时不会重复 trigger；已存在 worker 仍会 trigger。
- [x] Rust 测试覆盖 passive trigger 旧入口不启动 one-shot，manual 仍可 one-shot。
- [x] Rust 单元测试覆盖 Gmail History status 检查和分页收集逻辑。
- [x] 前端测试覆盖 `mail:new` 快速多次到达时只进行一次合并刷新。
- [x] 前端测试覆盖 SSE 重连后触发 catch-up，首次连接不触发。
- [x] 现有前端/Rust 相关测试通过。
- [x] `cargo fmt --check`、针对修改 crate 的 `cargo test`、前端相关测试通过；如全量门禁无法在时间内完成，需记录已跑和未跑原因。

## Definition of Done

- 先写或调整严格测试，再写实现。
- 每个建议至少有对应自动化测试或明确手动/命令验证。
- 不把用户已有未识别脏文件纳入提交。
- 若行为契约变化，更新 `.trellis/spec/pebble/backend/webmail-api-contracts.md` 或记录无需更新的理由。

## Technical Approach

### 数据流

Provider → SQLite store → `StoredMessage` → rules/indexing pipeline → SSE/Web Push → frontend SSE client → React Query invalidate → REST API refetch。

### 实施顺序

1. 后端 `mail:new` 最终状态：先补 `indexing.rs` 测试，再移动 emit 时机并构造最终 payload。
2. 前端刷新合并：抽出可测试的 coalescer/hook 行为或用组件测试验证 `StatusBar` 的 invalidate 次数。
3. wake 去重：让 `start_sync_inner` 返回启动状态，`wake_sync` 根据 `Started/AlreadyRunning` 决定是否投递 trigger。
4. SSE catch-up：扩展 `sse-client.ts` 支持 reconnect callback；`Layout` 或专用 hook 在重连后 wake + invalidate。
5. Gmail History：抽出分页收集/response 处理函数，增加单元测试，再接入 `poll_changes`。
6. 旧 trigger 收紧：按 `SyncTrigger::should_sync_now()` 控制缺 worker one-shot。

## Decision (ADR-lite)

**Context**：当前重复请求主要由事件语义不清和刷新边界过宽引起；同时断线期间 SSE 事件可能丢失。  
**Decision**：保留 push + poll + reconcile 的组合，但明确 poll completion 是心跳，数据变化由最终状态事件驱动；前端对事件合并刷新，断线后主动 catch-up。  
**Consequences**：实现复杂度略增，但能同时保证及时性、降噪和离线恢复。完整 event seq/replay 暂不纳入本轮 MVP，先用 reconnect catch-up 兜底。

## Out of Scope

- 不在本轮实现持久化 SSE event log / `changes?since=seq`。
- 不重命名公开事件 `mail:new`，避免前后端大面积迁移；本轮只修正其语义和 payload。
- 不重构 shell 为 static/dynamic 两个接口；本轮只减少触发频率。
- 不实现 Outlook webhook。

## Technical Notes

- 参考审计：`research/current-pipeline-audit.md`。
- 关键后端文件：`server/src/rpc/indexing.rs`、`server/src/rpc/sync_cmd.rs`、`server/src/gmail_realtime.rs`、`crates/pebble-mail/src/gmail_sync.rs`、`crates/pebble-mail/src/provider/gmail.rs`。
- 关键前端文件：`src/lib/sse-client.ts`、`src/components/StatusBar.tsx`、`src/app/useRealtimeSyncTriggers.ts`、`src/app/Layout.tsx`、`src/hooks/queries/useShellQuery.ts`。
- 相关契约：`.trellis/spec/pebble/backend/webmail-api-contracts.md`。
