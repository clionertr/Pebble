# Pebble Webmail 性能与用户体验全面审计报告 (AI 编码执行版)

> **元数据**
> * **生成日期**: 2026-05-16
> * **目标分支**: `001-improve-experience`
> * **阅读对象**: 编码辅助 AI (Coding Agent / Copilot)
> * **核心意图**: 指导代码重构，消除系统性延迟，优化资源分配，提升前端渲染效率与用户体验。
> * **Phase 0 (延迟诊断基线)**: 已完成，commit `5f9a8c9`

---

## 🧠 系统心理图像 (System Mental Model)

在进行代码修改前，请建立以下架构直觉：
Pebble 目前是一个"拥有顶级硬件设施，但调度系统混乱的现代物流枢纽"。

* **高速通道被低效调度阻塞**：Gmail Pub/Sub 和 IMAP IDLE 提供了极快的新邮件推送（救护车/特快专递），但底层断路器（Circuit Breaker）和同步队列将它们与普通的后台轮询（重型卡车）混在一起。一旦发生轻微网络波动，断路器会无差别拦截所有请求，导致实时推送失效。
* **过度劳动的仓储更新**：每次有单点数据变更（如收到一封新邮件），前端 React Query 会引发"掀桌子"式的全量列表刷新；后端则会在 Tantivy 搜索引擎和 SQLite 数据库中维持双倍的完整载荷（正文双存）。
* **自动化流水线上的"人工卡点"**：在完全异步的 Tokio 运行时中，存在致命的同步文件读取（如 SMTP 发送附件），这相当于在高速运转的传送带上强行要求人工查验，直接卡死 Worker 线程。

---

## 🚀 核心行动指南：按优先级排序的代码重构任务

### 🔴 P0 级别：紧急修复 (阻断性与核心体验破坏)

这部分需要在第一阶段立刻执行，修改成本低但收益极大。

| 任务 ID | 任务描述 | 目标文件 | 修复策略 |
| --- | --- | --- | --- |
| p0-sync-blocking-fix | **消灭异步上下文中的同步阻塞** | `crates/pebble-mail/src/smtp.rs:117` | 将 `std::fs::read` 替换为 `tokio::fs::read`，或使用 `tokio::task::spawn_blocking` 包裹，防止阻塞 Tokio Worker 线程。 |
| p0-push-bypass-circuit | **实时推送绕过断路器** | `crates/pebble-mail/src/backoff.rs` `crates/pebble-mail/src/idle.rs` | 为 `ProviderPush` (来自 IDLE 或 Webhook) 提供特权通道。即使主轮询断路器打开，推送触发的局部同步也应以"半开"或"绕过"模式执行。 |
| p0-sse-reconnect | **前端 SSE 断线重连与丢弃监控** | `src/tauri-mock.ts` `src-tauri/src/main.rs` | 前端为 `EventSource` 补充 `onerror` 处理器，实现指数退避重连。后端监控并记录 `broadcast::channel` 丢弃的事件。 |
| p0-remove-splash-block | **移除人为闪屏阻塞** | `src/App.tsx:58-69` | 移除强制的 1200ms 最低等待时间和 500ms 渐出动画。当应用状态就绪时，立即卸载闪屏。 |
| p0-a11y-upload-input | **修复严重 A11y 缺陷** | `src/features/compose/ComposeView.tsx:450-452` | 移除文件上传 `<input>` 的 `tabIndex={-1}` 和 `aria-hidden="true"`，或确保有一个可通过键盘触发的关联 `<button>`。 |

### 🟠 P1 级别：架构与数据流优化 (高收益性能改进)

这部分需要调整数据流转方式和缓存策略。

| 任务 ID | 任务描述 | 目标文件 | 修复策略 |
| --- | --- | --- | --- |
| p1-cache-patching | **停止前端全局刷新风暴** | `src/components/StatusBar.tsx` `src/lib/api.ts` | 收到 `mail:new` 后，**不要**粗暴调用 `refetchQueries`。应使用 `queryClient.setQueryData` 进行精准的缓存修补 (Cache Patching)，并将其他非活跃视图的重算进行防抖 (Debounce) 或标记失效。 |
| p1-composite-indexes | **消除数据库全表扫描** | `crates/pebble-store/src/migrations.rs` (或对应迁移文件) | 补充缺失的复合索引，特别是 `(account_id, is_deleted, date DESC)` 和 `(date DESC, id ASC)`，以优化列表分页。 |
| p1-tantivy-dedup | **Tantivy 索引减负 (消除双存)** | `crates/pebble-search/src/schema.rs:176` | `body_text` 仅保留 `INDEXED`，移除 `.set_stored()`。在展示搜索结果或片段时，回退到 SQLite 提取 snippet，节约一半以上的磁盘空间。 |
| p1-split-zustand-store | **拆解 Zustand God Store** | `src/stores/ui.store.ts` | 当前任何属性更改都会导致大面积重绘。按关注点拆分为独立 Store（如 `useThemeStore`, `useSyncStore`, `useLayoutStore`）。 |
| p1-imap-connection-pool | **IMAP 连接复用** | `src-tauri/src/rpc/messages/lifecycle.rs` | 改变当前每次归档/删除操作都重新建立完整 TCP/TLS 连接的逻辑，实现跨请求的 IMAP 连接复用 (Connection Pool)。 |

### 🟡 P2 级别：质量、一致性与深度打磨

| 任务 ID | 任务描述 | 目标文件 | 修复策略 |
| --- | --- | --- | --- |
| p2-rpc-timeout-concurrency | **后端 RPC 并发与超时控制** | `src-tauri/src/main.rs` | 在 Axum 路由层引入 `tower::TimeoutLayer` 和 `ConcurrencyLimitLayer`。防止慢请求雪崩。 |
| p2-thread-virtualization | **长对话线程渲染虚拟化** | `src/features/inbox/ThreadView.tsx` | 长对话目前是全量渲染，需引入 `@tanstack/react-virtual` 或默认折叠历史消息。 |
| p2-n1-query-optimization | **N+1 查询优化** | `crates/pebble-store/src/rpc/messages/lifecycle.rs` | 重构 `resolve_message_context`，将循环内的 4+ 次独立查询合并为单次 `JOIN` 查询或批量 `IN` 查询。 |
| p2-search-box-activation | **搜索框激活** | `src/features/inbox/InboxView.tsx:114` | 为当前为空函数的 `onSearch` 和 `onClear` 绑定实际的过滤或全局搜索逻辑，否则将其移除。 |
| p2-deferred-storage-persist | **延迟状态持久化** | `src/stores/ui.store.ts` | 避免在状态 Setter 中同步调用 `localStorage.setItem`，应使用 `requestIdleCallback` 或防抖函数将其移出主渲染线程。 |

---

## ⏱️ 关键硬编码参数速查表 (供调参参考)

在重构调度策略时，请注意以下关键的魔法数字已被硬编码，可能需要提取为常量或移入配置文件：

### 后端与协议层

* **IMAP IDLE 最小等待**: `60s` (`idle.rs:26`) - *限制了实时反馈的下限*
* **活跃状态衰减窗口**: `60s` (`sync.rs:1599`) - *前台转空闲过快*
* **默认前台空闲轮询**: `≥30s` (`realtime_policy.rs:16`)
* **断路器熔断阈值**: `5次连续失败` (`backoff.rs:17`)
* **断路器最大退避时间**: `300s` (`backoff.rs:16`)
* **Gmail 推送防抖合并窗口**: `30s` (`gmail_realtime.rs:29`) - *可考虑缩短至 10-15s 或按 historyId 进行去重*
* **SSE 广播通道容量**: `100` (`state.rs:50`) - *过小，且溢出时静默*
* **Tantivy Writer 内存缓冲**: `50MB` (`lib.rs:127`) - *对低内存 VPS 不友好*
* **RPC 批处理合并窗口**: `50ms` (`tauri-mock.ts`)

### 前端与 UI 层

* **React Query Stale Time**: `30s` (`query-client.ts:6`)
* **闪屏硬性最小阻塞**: `1200ms + 500ms fade` (`App.tsx:58-69`) - *P0 级待除*
* **搜索防抖**: `300ms` (`SearchView.tsx:94`)

---

## 📋 定义与参考

### Definition of Done (团队质量门槛)
- 测试已添加/更新。
- `cargo check -p pebble`、Rust 测试、前端构建/类型检查全部通过。
- 生产发布说明包含日志标志、测量步骤、回滚路径、前后对比指标。
- 如果用户可见的设置或操作行为发生变化，文档/笔记已更新。

### Out of Scope (明确排除)
- 替换 Gmail API + Pub/Sub 为其他提供商机制。
- 移除 IMAP 支持或破坏现有 IMAP 登录/账户行为。
- 重新设计整个 UI 视觉系统。
- 构建通用分布式任务系统（除非测量结果表明当前进程内调度器无法满足目标）。

### Technical Notes (关键代码路径)
- `src-tauri/src/gmail_realtime.rs` — Gmail Pub/Sub webhook 处理、推送状态持久化、watch 续期、推送合并、触发 provider_push 同步。
- `src-tauri/src/rpc/sync_cmd.rs` — 同步 worker 生命周期、通过无界通道发送触发信号。
- `crates/pebble-mail/src/gmail_sync.rs` — Gmail History API 增量同步、`buffer_unordered(10)` 并发获取。
- `crates/pebble-mail/src/sync.rs` — IMAP IDLE + 轮询回退。
- `crates/pebble-mail/src/realtime_policy.rs` — 触发行为定义。ProviderPush 立即同步，WindowBlur 仅更新运行时状态。
- `src/app/useRealtimeSyncTriggers.ts` — 在 focus/blur/network-online 时触发所有账户。
- `src-tauri/src/rpc/indexing.rs` — 发出 `mail:new`、应用规则、索引新消息、批量提交搜索。
- `src/components/StatusBar.tsx` — 处理 `mail:new` 事件，刷新 messages/threads 查询。
- `src/lib/mailLatencyLogging.ts` — Phase 0 新增，前端延迟日志模块。
- `src-tauri/src/mail_latency.rs` — Phase 0 新增，后端延迟日志核心模块。
