# brainstorm: sync upstream changes

## Goal

将 `QingJ01/Pebble` 上游在桌面邮件客户端方向积累的高价值修复，按 Webmail 项目的架构边界选择性同步到当前项目。同步方式应优先保护现有 Webmail 形态：Axum HTTP 服务、React SPA、REST/SSE 客户端、Docker/自托管部署和移动端适配。

## What I already know

* 当前项目从 `QingJ01/Pebble` fork 而来，已经从 Tauri 桌面客户端改造成 Webmail。
* 当前仓库有 `upstream = https://github.com/QingJ01/Pebble.git`，刷新后 `upstream/master` 位于 `10e65bb docs: add Pebble Web section to README`。
* 当前本地 `master` 位于 `d45d928 chore: archive webmail migration task, remove dead code, record session 7`。
* 最近共同祖先是 `b7fac6b3f9616b19fd1077cf60948289621e17d2`，两边已经明显分叉。
* 不能整体 merge `upstream/master`：按当前内容差异，上游没有本项目的 `server/`，并会删除 Webmail 的认证、REST/SSE 客户端等关键文件。
* 本轮更适合做“按补丁摘取”：先同步共享 Rust crates，再考虑前端可移植逻辑，最后处理文档/站点与非目标内容。
* 用户明确列出不应合入的内容：`src-tauri/`、Docker 不适合的 OS Keyring 切换、Tauri 原生更新/托盘/关闭逻辑、移除移动布局、移除所有账户视图、移除 Sidebar 移动逻辑、删除 `useDelayedIdleReady`。
* 上游刷新后比用户清单多出几类内容：README/Web 说明、站点展示、自定义背景图、Sidebar 视图切换卡住修复。
* 用户已选择本轮同步边界为“Crates + 精选前端修复”：先同步共享 Rust crates 的稳定性修复，再移植确实适合 Webmail 的前端修复。
* 用户已选择移植策略为“补缺优先”：本地已有 Webmail 适配就保留，只补明确缺口，不用上游实现整段替换本地实现。
* 代码扫描发现部分前端能力当前已经存在或部分存在，例如 `ComposeView.tsx` 已有粘贴图片附件逻辑，`MessageList.tsx` 已有批量操作后刷新未读计数的逻辑。
* `git cherry -v master upstream/master` 显示大量上游提交仍不是 patch-id 等价补丁，但当前项目也有 `fix: improve mail sync reliability`、`fix: merge upstream frontend mail UX fixes`、`fix: use global proxy for translation` 等本地同主题提交，后续必须逐项判重。
* 当前 Webmail OAuth 已通过 `server/src/auth.rs` 的 `/auth/login` 和 `/auth/callback` 处理，前端 `startOAuthLogin()` 跳转到该网页登录入口。
* 上游 `pebble-oauth` 新增的本地 TCP 回调监听器绑定 `127.0.0.1:{port}/callback`，更符合桌面应用语义；在 Docker/Webmail 中，`127.0.0.1` 指向服务端容器或宿主机，不一定是用户浏览器所在机器。
* 用户已决定排除上游 `pebble-oauth` 的本地 TCP redirect listener，保留当前 Webmail 的服务器回调模型。
* 上游 `pebble-store` 的消息列表 JOIN 优化不能直接照搬：当前项目已有稳定分页排序、重复消息去重和相关测试，直接采用上游 diff 会削弱这些语义。
* `pebble-store` 实际 diff 主要包含 4 个文件：`cloud_sync.rs`、`messages.rs`、`migrations.rs`、`pending_ops.rs`。
* 上游 `cloud_sync.rs` 新增 WebDAV 备份 payload shape 校验：拒绝空响应、HTML/XML 目录页、非 JSON object 响应，并把备份上传 Content-Type 改为 `application/json`；这是小范围且适合 Webmail 的补丁。
* 上游 `messages.rs` 将文件夹消息列表从 `EXISTS` 改为 `JOIN`，但同时移除了当前项目已有的稳定分页排序 `ORDER BY m.date DESC, m.id ASC` 和多文件夹去重/删除过滤测试；不能原样合入。
* 当前项目已有 `get_message_with_provider()`，server 的消息生命周期代码正在使用；上游 diff 会删除它，必须排除。
* 当前项目 schema 版本为 13，包含 `idx_messages_account_deleted_date` 和 `idx_messages_deleted_date_id` 两个索引迁移；上游仍是 11，不能让迁移版本回退。
* 当前项目 `pending_ops.rs` 有 `delete_pending_mail_op()`，server API 正在使用；上游 diff 会删除它，必须排除。
* 用户已决定 `pebble-store` 本轮只合入 WebDAV 备份校验和 `application/json` 上传；消息列表查询语义保留当前实现，不做 JOIN 重写。
* 上游 `pebble-search` 将 `body_text` 改为 Tantivy stored field，并直接从索引生成 snippet；当前项目则让搜索命中后再从 SQLite 读取正文片段。
* 当前 Webmail server 没有像上游 Tauri 启动流程那样自动检查 `search.needs_reindex()` 并后台重建索引；若采用 stored body schema，需要补上 Webmail 启动/后台重建策略。
* 用户已决定采用上游 `pebble-search` stored body_text 方案，但要求 Webmail server 补上启动后的后台 reindex 检查，避免用户手动修复索引。
* `pebble-search` 实际上游 diff 只改 `crates/pebble-search/src/schema.rs` 和 `crates/pebble-search/src/lib.rs`：`body_text` 从 indexed-only 改为 stored，`schema_needs_rebuild()` 要求 `body_text` stored，并新增直接从 Tantivy doc 截取 150 字符 snippet 的逻辑。
* 当前 `server/src/rpc/search.rs` 和 `server/src/rpc/advanced_search.rs` 在搜索后回查 SQLite 补 `snippet`，这是为了适配当前 indexed-only body schema；采用 stored body 后应删除这段二次查库。
* 当前 `server/src/main.rs` 只打开 `TantivySearch::open(&index_path)`，没有处理 `search.needs_reindex()`、`doc_count() == 0 && store.count_all_messages() > 0` 或 `doc_count() != store.count_all_messages()` 等情况。
* 当前已有 `server/src/rpc/indexing.rs::do_reindex()` 和 `server/src/rpc/sync_cmd.rs::reindex_search()`，可以复用为启动自修复入口，不需要新增一套重建索引逻辑。
* 当前 `pebble-store` 已有 `count_all_messages()`，可用于启动时判断索引文档数量和数据库非删除消息数量是否一致。
* 用户已决定 Webmail server 启动时采用后台 reindex：检测到 schema 变更、索引为空但数据库有邮件、或索引数量与数据库非删除消息数量不一致时，服务先启动并在后台重建索引。
* 邮件同步 diff 中存在 Webmail 专用能力冲突：上游 `GmailProvider` 将 `has_push` 设为 `false` 并移除 watch/stop watch 入口，而当前项目已经有 `server/src/gmail_realtime.rs` 的 Gmail Pub/Sub 实时同步。
* 上游 Gmail history 修复位于 `crates/pebble-mail/src/gmail_sync.rs`，但必须按补缺方式合入，不能回退当前 Webmail 的 Gmail Pub/Sub 支持。
* 用户已决定 Gmail 相关同步修复采用“保留 Webmail Pub/Sub，只补 history/pagination/错误处理缺口”的策略。
* `pebble-mail` 中 IMAP TLS 1.2、Tencent enterprise mail 兼容、Outlook move 后 remote_id 更新、IMAP UID baseline、rustls unexpected EOF retry 等上游主题在当前项目中已有同主题本地提交或等价实现；后续以测试验证和局部补缺为主。
* `pebble-mail` 当前剩余 diff 主要集中在 5 个文件：`gmail_sync.rs`、`provider/gmail.rs`、`realtime_policy.rs`、`smtp.rs`、`sync.rs`。
* 上游剩余 `provider/gmail.rs` diff 会移除 `GmailWatchResponse`、`watch_inbox()`、`stop_watch()`，并把 `has_push` 改为 `false`；这些与 Webmail Gmail Pub/Sub 冲突，必须排除。
* 上游剩余 `realtime_policy.rs` / `sync.rs` diff 会让 `ProviderPush` 不再绕过 circuit breaker，并在 IMAP IDLE provider push 时若 circuit open 就忽略本次推送；这可能降低错误风暴，但也会降低 Gmail Pub/Sub/IMAP IDLE 的实时恢复能力。
* 上游剩余 `smtp.rs` diff 把附件读取从 `tokio::fs::read` 改回 `std::fs::read`；当前 Webmail server 运行在 async runtime 中，保持异步读取更合适，默认不采用。
* 上游剩余 `gmail_sync.rs` diff 改变分页 limit 语义：当前项目限制每个 label 的总抓取数量，上游则每页都使用同一 limit 并继续翻页；需要在 Gmail history/initial sync 补缺时判断是否采用“抓完整分页”的行为。
* 用户已决定采用上游 `ProviderPush` circuit breaker 保护：推送触发不再绕过 backoff；连续失败时忽略推送直到对应账号的 backoff 恢复。
* `ProviderPush` backoff 必须按账号隔离：一个账号 circuit open 只能压住该账号的 Gmail Pub/Sub 或 IMAP IDLE 推送，不能影响其他账号。
* 上游 Store 架构整合不是孤立改动：它删除 `theme.store.ts`、`sync.store.ts`，把 theme/sync/background/notifications 合并进 `ui.store.ts`，并连带 Tauri 事件、自定义背景图、关闭到后台等桌面逻辑。
* 上游 `Layout.tsx` 相关 diff 会移除当前 Webmail 的 `AuthProvider`/`LoginView` 路径、替换 SSE 为 Tauri `listen`，并削弱移动抽屉逻辑，因此不能直接套用。
* 用户已决定本轮排除 Store 架构整合：保留当前 `ui.store.ts` + `theme.store.ts` + `sync.store.ts` 分层。
* 前端候选中，`ComposeView.tsx` 的粘贴图片附件和 `TranslateTab.tsx` 的真实错误信息提取已经在当前项目中等价存在，后续只需验证。
* 上游 `InboxView.tsx` diff 会删除当前 Webmail 的“所有账户”视图和移动端列表/详情切换逻辑，不能合入。
* 上游 `MessageList.tsx` diff 会删除当前项目的延迟标签加载和 mail latency logging，不符合 Webmail 性能优化方向。
* 上游 `29a8ab0` 的 Sidebar 卡住修复可用，但只能移植 `safeSetActiveView` / `handleFolderClick` 的最小确认逻辑，不能带入移除移动逻辑、改 store 架构等关联变更。
* 用户已决定体验类内容本轮只纳入 Sidebar 卡住修复；自定义背景图、站点展示、README Web 展示先排除。
* `Sidebar` 卡住修复的核心是 `Sidebar.tsx` 只在草稿真的脏时才 await 确认；实现验证发现 `ui.store.ts::setActiveView()` 仍被快捷键、命令面板、搜索入口等非 Sidebar 路径依赖，用于全局脏草稿保护，因此本轮保留该 guard。
* 当前 `ComposeView.tsx` 已有粘贴图片作为附件逻辑，但隐藏 file input 缺少上游新增的 `aria-hidden="true"`；这是小范围可直接合入的无障碍修复。
* 上游 `ShadowDomEmail.tsx` / `useMailtoOpen.ts` / `features/compose/mailto.ts` 依赖 Tauri `invoke` / `listen` 来打开外部 URL 和处理系统级 `mailto:`，不能直接合入 Webmail；第 9 条已决定不在本轮补站内 `mailto:` compose。
* 用户已决定前端本轮纳入 Sidebar 卡住修复和 Compose 隐藏 file input `aria-hidden="true"`；继续排除 Webmail 版 `mailto:` 站内写信。
* `pebble-translate` 全局代理能力当前已经存在：`server/src/rpc/translate.rs` 读取全局代理并调用 `TranslateService::translate_with_proxy()`。
* 上游 `pebble-translate` 的 LLM 参数改动主要是内部函数签名变化，将当前项目已有的 `LlmTranslateRequest` 结构拆成多个参数；它不是明显的用户功能补缺。
* 用户已决定排除 `pebble-translate` 的 LLM 签名重构：保留当前 `LlmTranslateRequest` 结构，只验证全局代理和错误显示已覆盖。
* `pebble-privacy` 中，当前项目已经等价吸收大部分隐私渲染修复：受信任发件人模式、预置头隐藏样式保留、邮件布局保留、full document body fragment 提取、安全 background shorthand 处理等能力都已存在。
* `pebble-privacy` 的主要缺口是上游新增的 `PrivacyGuard::render_message_html(raw_html, body_text, mode)`：当 HTML 为空但纯文本正文存在时，生成安全的 `<pre class="pebble-plain-text-email">...</pre>`；同时对 HTML 文本节点里的裸 `http(s)://` URL 和邮箱地址自动转成链接，并避免重复处理已有 `<a>`。
* 当前 Webmail server 的 `server/src/rpc/messages/rendering.rs` 仍调用 `guard.render_safe_html(&message.body_html_raw, &effective_mode)`，因此纯文本-only 邮件可能渲染为空，裸 URL/邮箱也不会变成可点击链接。
* 上游 `ShadowDomEmail` 链接处理依赖 Tauri `invoke("open_external_url")`，不能直接搬到 Webmail；若后端把邮箱 linkify 成 `mailto:`，前端应采用 Webmail 适配策略，而不是引入 Tauri 调用。
* 用户已决定采用 `pebble-privacy` 的后端纯文本渲染和裸 URL/邮箱自动链接化，但前端不引入 Tauri 链接打开逻辑；`mailto:` 暂走浏览器默认行为。

## Assumptions (temporary)

* 共享 crates 层的邮件同步、搜索、存储、隐私渲染、翻译和 OAuth 修复整体优先级高于桌面 UI 改动。
* 前端同步需要从 `invoke()` 语义转换到当前 `src/lib/api.ts`、`src/lib/api-client.ts`、React Query 和 SSE 刷新模型。
* `pebble-crypto` 的 OS Keyring 切换默认排除，除非后续专门设计 Webmail/Docker 兼容的密钥后端策略。
* README/站点中关于 Pebble Web 的上游内容可能需要人工改写，而不是直接接受上游文案。

## Open Questions

* Gmail 初始同步分页 limit 是否改为抓完整分页，还是保留当前每个 label 的总量上限。

## Requirements (evolving)

* 采用选择性 cherry-pick / 手工移植，不做 `upstream/master` 整体 merge。
* 保留当前 Webmail 架构和部署路径。
* 优先同步用户列出的 crates 层高价值修复。
* 对前端改动逐项评估是否需要 REST/SSE 适配。
* 对每个候选上游补丁先做等价能力判重，避免重复移植或回退本地 Webmail 专用实现。
* 对已经部分存在的能力采用补缺优先策略：保留当前 Webmail 适配，只补明确缺口。
* 排除上游 `pebble-oauth` 的本地 TCP redirect listener，不新增 `127.0.0.1:{port}/callback` 监听流程。
* `pebble-store` 查询优化只能在保留当前分页稳定性、去重语义和测试的前提下重写，不直接套用会上游删除本地测试的版本。
* `pebble-store` 必须保留 schema version 13、V12/V13 索引迁移、`get_message_with_provider()` 和 `delete_pending_mail_op()`。
* `pebble-store` 本轮排除消息列表 JOIN 重写；保留当前 `EXISTS` 查询、稳定分页排序和多文件夹去重测试。
* `pebble-store` 本轮纳入 WebDAV payload shape 校验，提升远程备份导入错误提示。
* 采用 `pebble-search` stored body_text 时，必须同步实现 Webmail server 后台 reindex 检查：schema 变化、索引为空但数据库有消息、索引数量和数据库消息数量不一致时自动重建。
* `pebble-search` 自动 reindex 不阻塞 HTTP 服务启动；重建失败只记录错误，不能导致 Webmail 进程退出。
* Gmail 相关补丁不得移除当前 Webmail 的 Pub/Sub 实时同步、watch/stop watch 辅助能力或 `ProviderPush` 事件通路。
* Gmail history、分页、错误处理等修复按补缺方式合入，禁止通过跟随上游桌面版禁用 push 来降低冲突。
* `pebble-mail` 已存在的 IMAP/TLS/Tencent/Outlook/UID baseline 修复不做整段覆盖；只在测试证明有缺口时补局部逻辑。
* `ProviderPush` 采用 circuit breaker 保护时，backoff 状态必须由各账号 worker 自己维护，禁止引入全局推送熔断。
* 前端移植不得移除当前 Webmail 认证、SSE、移动端抽屉、所有账户视图或延迟加载策略。
* Sidebar 卡住修复必须保留 Webmail `isMobile` / `drawerOpen` 行为；`ui.store.ts` 的全局脏草稿保护继续保留，避免非 Sidebar 导航静默丢草稿。
* 排除 Store 架构整合，不合并 `theme.store.ts`、`sync.store.ts` 到 `ui.store.ts`，除非后续单独开前端状态架构任务。
* `ComposeView.tsx` 粘贴图片附件、`TranslateTab.tsx` 真实错误信息、`MessageList.tsx` 批量操作刷新未读计数等已存在能力，默认不重复移植，只保留/补测试。
* 体验类前端只移植 Sidebar 卡住修复和 Compose `aria-hidden`；自定义背景图、站点、README 展示、`mailto:` 站内写信另开任务处理。
* 排除 `pebble-translate` LLM 函数签名重构，不把当前结构化请求对象退回多参数调用。
* `pebble-privacy` 已存在能力不重复移植；本轮纳入后端 `render_message_html(raw_html, body_text, mode)`、纯文本 fallback 和裸 URL/邮箱自动链接化。
* 前端邮件链接处理不引入 Tauri `invoke("open_external_url")`，`mailto:` 站内 compose 行为不作为本轮要求。
* 明确记录排除项，避免后续误把桌面专属逻辑带回项目。

## Acceptance Criteria

* [x] 得到明确同步范围和排除范围。
* [x] 为每一类待同步内容标注来源、目标文件、移植方式和验证方式。
* [x] 同步后 `cargo test` / 前端测试 / 类型检查按影响范围通过。
* [x] 不恢复 `src-tauri/` 作为运行路径，不破坏 `server/` 和 REST/SSE API。
* [x] 不删除移动端适配、所有账户视图或延迟加载优化。

## Definition of Done (team quality bar)

* Tests added/updated (unit/integration where appropriate)
* Lint / typecheck / CI green
* Docs/notes updated if behavior changes
* Rollout/rollback considered if risky

## Out of Scope (explicit)

* 整体合并上游桌面应用。
* 恢复 Tauri 桌面运行时。
* 直接采用 OS Keyring 作为 Webmail/Docker 默认密钥存储。
* 上游 `pebble-oauth` 本地 TCP redirect listener，除非后续专门设计本机部署/桌面模式。
* 上游自定义背景图、站点展示、README Web 展示。
* 删除 Webmail 移动端、所有账户视图、REST/SSE 客户端和 Axum 服务。

## Technical Notes

* 已执行 `git fetch upstream --prune`，上游最新为 `10e65bb`。
* `git diff master upstream/master -- server src` 显示整体合并会删除 `server/` 与多处 Webmail 前端基础设施。
* `git diff master upstream/master -- crates` 显示 crates 层仍存在真实内容差异，适合作为第一批候选。
* 用户给出的上游差异清单可作为初始候选，但需要用实际 diff 校验每项是否已经在本项目中等价实现。
* 已实现 `pebble-privacy` 纯文本 fallback 与裸 URL/邮箱 linkify，并让 Webmail 消息渲染入口统一调用 `render_message_html()`。
* 已实现 `pebble-search` stored `body_text` snippet，并在 Webmail server 启动时按 schema 变化、空索引或数量不一致触发后台 reindex。
* 已实现 `pebble-store` WebDAV 备份 payload shape 校验与 `application/json` 上传。
* 已实现 `pebble-mail` ProviderPush circuit breaker 保护，熔断状态仍按账号 worker 隔离。
* 已实现前端 Sidebar 脏草稿条件确认与 Compose 隐藏文件输入 `aria-hidden`；保留 `ui.store.ts` 全局脏草稿保护。
* 已通过 `pnpm test`、`pnpm run build:frontend`、`cargo fmt --check`、`cargo clippy --all-targets -- -D warnings`、`cargo test --all`。
