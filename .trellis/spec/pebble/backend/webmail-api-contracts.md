# Webmail API 契约

## Scenario: 标准 Webmail REST/SSE 边界

### 1. Scope / Trigger
- Trigger: Pebble 已从桌面 JSON-RPC 迁移为浏览器 Webmail；认证、附件、实时事件、部署和前端调用方式都跨越前后端边界。
- 范围：`server/src/api/`、`server/src/middleware/`、`src/lib/api-client.ts`、`src/lib/sse-client.ts`、Docker/nginx 部署。

### 2. Signatures
- 登录：`POST /api/auth/login`，body `{ "password": string }`。
- 会话状态：`GET /api/auth/status`。
- 登出：`POST /api/auth/logout`。
- SSE：`GET /events`，必须带 `pebble_session` cookie。
- OpenAPI：`GET /api/docs` 和 `GET /api/docs/openapi.json`。
- 附件上传：`POST /api/attachments/stage`，`multipart/form-data`。
- 附件下载：`GET /api/attachments/:id`，返回流式 body。

### 3. Contracts
- Cookie：登录成功设置 `pebble_session`，属性必须包含 `HttpOnly; Secure; SameSite=Strict; Max-Age=604800`。
- 鉴权：除 `/api/auth/login`、`/api/auth/logout`、`/api/auth/status`、`/api/docs*`、`/auth/*`、`/webhook/gmail` 外，`/api/*` 和 `/events` 必须要求有效 session。
- 环境变量：`PEBBLE_PASSWORD_HASH` 必填；`ALLOWED_ORIGIN` 仅在前后端跨域部署时设置；Docker Compose 负责设置 `PEBBLE_HOST=0.0.0.0`。
- 前端：生产代码不得调用 `invoke(...)`、`/rpc` 或 Tauri mock；HTTP 调用集中在 `api-client.ts`/`api.ts`。
- 部署：nginx 只代理 `api|events|auth|webhook`，不得重新暴露 `/rpc`。

### 4. Validation & Error Matrix
- 缺少 `PEBBLE_PASSWORD_HASH` -> 进程启动失败。
- 登录密码为空 -> `400`。
- 登录密码错误 -> `401`。
- 登录失败超过 5 次 -> `429`，按来源 IP 锁定 15 分钟。
- 无 session 访问 `/api/*` 或 `/events` -> `401`。
- 附件不存在 -> `404`。
- Gmail webhook secret 错误 -> `401`。
- Gmail webhook payload 非法 -> `400`。

### 5. Good/Base/Bad Cases
- Good: 浏览器登录后，REST 请求和 EventSource 都自动携带 cookie，收件箱和实时事件都可用。
- Base: 同源 Docker/nginx 部署中 `ALLOWED_ORIGIN` 为空，CORS 不放宽，前端通过 nginx 访问后端。
- Bad: 在前端新增 `invoke("send_email")` 或在 nginx 恢复 `/rpc` 代理，会绕开 Web API 契约和测试保护。

### 6. Tests Required
- Rust API 测试：登录 cookie 属性、登出失效、`/events` 未登录 401、认证豁免路由可访问。
- Rust inventory 测试：前端无 `invoke(`、无 `/rpc`、CI 不构建 Tauri/desktop 包。
- 前端测试：API client 请求路径和 snake_case/camelCase 契约；登录门禁阻止未认证状态下启动数据请求。
- 部署检查：`cargo clippy --all-targets -- -D warnings`、`cargo test --all`、`pnpm test`、`pnpm build:frontend`。

### 7. Wrong vs Correct

#### Wrong
```typescript
await invoke("update_account_proxy", { proxyHost, proxyPort });
```

#### Correct
```typescript
await apiPut("/api/accounts/account-1/proxy", {
  proxy_host: proxyHost,
  proxy_port: proxyPort,
});
```

## Scenario: 邮件隐私渲染查询参数

### 1. Scope / Trigger
- Trigger: 邮件详情页的“加载图片”“仅信任图片”“信任发件人”会跨越前端状态、HTTP 查询参数、后端渲染服务和 `pebble-privacy` 清理器。
- 范围：`src/lib/api.ts`、`src/hooks/useMessageLoader.ts`、`server/src/api/messages.rs`、`server/src/rpc/messages/rendering.rs`、`crates/pebble-privacy/`。

### 2. Signatures
- `GET /api/messages/:id/html?privacyMode=<mode>` -> `RenderedHtml`。
- `GET /api/messages/:id/full?privacyMode=<mode>` -> `[Message, RenderedHtml] | null`。
- 前端领域类型：`PrivacyMode = "Strict" | "LoadOnce" | "Off" | { TrustSender: string }`。
- HTTP 查询参数值：`strict`、`load_once`、`off`、`trust:<sender email>`。

### 3. Contracts
- 前端必须在 `src/lib/api.ts` 把领域枚举显式转换成 HTTP 参数；不要把 `PrivacyMode` 直接 `as string` 后传给 `api-client.ts`。
- `Strict`/`strict`：阻止外部图片和追踪图。
- `LoadOnce`/`load_once`：本次加载非追踪外部图片，仍阻止追踪像素和已知追踪域。
- `Off`/`off`：不做图片/追踪阻止。
- `trust:<sender email>`：只在参数邮箱与当前消息 `from_address` 匹配时按 `TrustSender` 渲染；不匹配必须回退到 `Strict`。
- 已持久化的 `TrustedSender(images)` 等价于 `LoadOnce`；`TrustedSender(all)` 等价于当前消息发件人的 `TrustSender`。
- 前端“加载图片”和未持久化的本次 `TrustSender` 覆盖必须绑定当前 `messageId`；切换到另一封邮件时必须回到全局默认隐私模式或该邮件自己的持久化信任结果。
- 隐私设置页取消信任时，按 `accountId + email` 删除 `trusted_senders` 记录；`images` 和 `all` 是同一发件人的不同信任等级，不是两条独立开关。

### 4. Validation & Error Matrix
- 缺少 `privacyMode` -> `Strict`。
- 未知 `privacyMode` -> `Strict`。
- `trust:<email>` 与当前消息发件人不匹配 -> `Strict`。
- `load_once` 下追踪像素或已知追踪域 -> 阻止并计入 `trackers_blocked`。
- `strict` 下普通外部图片 -> 替换为 blocked placeholder，并计入 `images_blocked`。
- A 邮件点击“加载图片”后切到 B 邮件 -> B 邮件不得沿用 A 的 `LoadOnce` 或 `TrustSender` 临时覆盖。
- 取消信任发件人或信任图片 -> 删除当前账号下该邮箱的整条信任记录，后续渲染不再获得持久化信任覆盖。

### 5. Good/Base/Bad Cases
- Good: 点击“加载图片”后，前端请求 `privacyMode=load_once`，后端立即重新渲染非追踪图片。
- Base: 点击“仅信任图片”后，即使持久化请求还没完成，本次渲染也可通过 `load_once` 显示普通图片；后续切换消息再由 `trusted_senders` 表生效。
- Bad: 前端发送 `privacyMode=LoadOnce` 或 `privacyMode=[object Object]`，后端把它兜底成 `Strict`，用户看到按钮无反应或必须切换界面才刷新。
- Bad: 前端把 `LoadOnce` 存成组件级全局状态，A 邮件授权后切到 B 邮件仍按 `LoadOnce` 首次渲染。

### 6. Tests Required
- 前端测试：断言 `getRenderedHtml("id", "LoadOnce")` 调用 HTTP client 时传入 `load_once`。
- 前端测试：断言 `{ TrustSender: "sender@example.com" }` 序列化为 `trust:sender@example.com`，不能变成 `[object Object]`。
- 前端测试：断言 A 邮件点击“加载图片”或“信任发件人”后，切到 B 邮件时 `useMessageLoader` 收到全局默认隐私模式。
- 前端设置测试：断言取消信任调用 `removeTrustedSender(activeAccountId, email)` 并从列表移除该邮箱。
- Rust API 测试：断言 `parse_privacy_mode` 识别 `strict/load_once/off/trust:*`，未知值回退 `Strict`。
- Rust 服务测试：断言 `TrustSender` 参数必须匹配当前消息发件人，不匹配时回退 `Strict`。
- `pebble-privacy` 测试：断言 `LoadOnce` 仍阻止追踪图，`TrustSender`/`Off` 才放开追踪图。

### 7. Wrong vs Correct

#### Wrong
```typescript
return client.getRenderedHtml(messageId, privacyMode as string);
```

#### Correct
```typescript
return client.getRenderedHtml(messageId, privacyModeQueryParam(privacyMode));
```

## Scenario: 邮件详情收件人展示语义

### 1. Scope / Trigger
- Trigger: 自动转发、别名收信或聚合多账号场景下，邮件“存在哪个账号”和“邮件头写给谁”可能不同；详情页不能把接收账户误当成邮件收件人。
- 范围：`src/components/MessageDetail.tsx`、`src/lib/api-types.ts`、`crates/pebble-core/src/types.rs`、`crates/pebble-mail/src/parser.rs`、`crates/pebble-mail/src/provider/gmail.rs`、`crates/pebble-mail/src/provider/outlook.rs`。

### 2. Signatures
- `GET /api/messages/:id/full?privacyMode=<mode>` -> `[Message, RenderedHtml] | null`。
- `Message.account_id: string`：本地存储/查询所属账号。
- `Message.to_list: EmailAddress[]`、`cc_list: EmailAddress[]`、`bcc_list: EmailAddress[]`：邮件头或 Provider API 返回的收件人列表。
- `EmailAddress = { name: string | null, address: string }`。

### 3. Contracts
- `account_id` 只表示“这封邮件同步到哪个 Pebble 账号”，不得作为 `To`、`Cc` 或 `Bcc` 的展示来源。
- 邮件详情页展示 `to ...` 时，必须优先使用 `message.to_list`；只有 `to_list` 为空时，才允许回退到账户邮箱作为兼容提示。
- 展示多个收件人时保持列表顺序，用逗号连接；有 `name` 和 `address` 时展示 `Name <address>`，只有 `address` 时展示邮箱地址。
- 同步/解析层如果能获得邮件头 `To`，必须写入 `Message.to_list` 并经 Store/API 原样传到前端。

### 4. Validation & Error Matrix
- `to_list` 非空且不等于接收账户 -> 详情页显示 `to_list`，不能显示接收账户。
- `to_list` 为空且能找到 `account_id` 对应账户 -> 详情页可显示接收账户作为 fallback。
- `to_list` 中存在空地址/空名称 -> 展示层跳过空展示项，避免渲染空白 `to` 文案。
- Provider 未返回原始收件人 -> 属于同步数据缺失；前端不得用 `account_id` 伪装成真实 `To`。

### 5. Good/Base/Bad Cases
- Good: `to_list=[Original <original@example.com>]` 且接收账户是 `receiver@example.com`，详情页显示 `to Original <original@example.com>`。
- Base: 老数据 `to_list=[]`，详情页显示 `to <receiver@example.com>`，用户仍能看到邮件属于哪个账号。
- Bad: 任何非空 `to_list` 场景都固定显示 `to <receiver@example.com>`，自动转发邮件会看起来像原本就发给接收账户。

### 6. Tests Required
- 前端组件测试：构造 `Message.to_list` 与 `useAccountsQuery()` 返回邮箱不同，断言 `MessageDetail` 显示 `to_list` 中的原始收件人。
- 前端组件测试：同一场景下断言接收账户邮箱不出现在 `to` 收件人位置。
- Rust 解析/Provider 测试：已有 IMAP/Gmail/Outlook 收件人解析用例应继续保证 `to_list` 从邮件头/API 字段填充。

### 7. Wrong vs Correct

#### Wrong
```tsx
const receivingAccount = accounts?.find((a) => a.id === message.account_id);
return <span>to&nbsp;&lt;{receivingAccount.email}&gt;</span>;
```

#### Correct
```tsx
const toRecipientDisplay =
  formatRecipientList(message.to_list) || (receivingAccount ? `<${receivingAccount.email}>` : "");
return toRecipientDisplay ? <span>to&nbsp;{toRecipientDisplay}</span> : null;
```

## Scenario: SSE 新邮件事件驱动前端缓存刷新

### 1. Scope / Trigger
- Trigger: 后端通过 `mail:new` 推送新邮件，前端使用 React Query 缓存消息列表、线程、文件夹、未读数和搜索结果。
- 范围：`server/src/rpc/indexing.rs`、`src/components/StatusBar.tsx`、`src/hooks/queries/useMessagesQuery.ts`、`src/hooks/queries/useThreadsQuery.ts`、`src/features/search/SearchView.tsx`。

### 2. Signatures
- SSE 事件：`event: mail:new`。
- Payload 字段：`account_id?: string`、`message_id?: string`、`folder_ids?: string[]`、`thread_id?: string | null`、`subject?: string`、`from?: string`、`received_at?: number`、`latency?: object | null`。
- 消息列表缓存键：`["messages", folderId, folderIds]`。
- 线程缓存键：`["threads", folderId, folderIds, limit, offset]`。
- 文件夹缓存键：`["folders", accountId]`。
- 未读数缓存键：`["folder-unread-counts", accountId]`。
- 搜索缓存键：`["search", query, filters]`。

### 3. Contracts
- 后端必须在规则处理、消息重载、最终 folder_ids 确定之后再发送 `mail:new`；不要在刚写入 provider 原始 folder_ids 后提前发送。
- `mail:new.folder_ids` 必须是规则处理后的最终文件夹集合；如果规则让消息无文件夹或消息已不可见，也要发送空数组事件，前端据此刷新列表，后端同时移除对应搜索文档。
- `mail:new.account_id` 是“事件来源账号”，不是消息/线程查询的 React Query key。
- 收到 `mail:new` 后，前端必须用 `["messages"]` 和 `["threads"]` 前缀失效列表缓存；不能用 `["messages", account_id]` 或 `["threads", account_id]`。
- 文件夹和未读数仍按账号精准失效：`["folders", account_id]`、`["folder-unread-counts", account_id]`。
- 快速连续收到多个 `mail:new` 时，前端必须合并刷新；当前约定是 500ms 窗口内只做一次 `["shell"]`、`["messages"]`、`["threads"]` 和账号级 folders/unread-counts 失效。
- 搜索索引由后端批量提交；前端收到 `mail:new` 后应延迟一次 `["search"]` 失效，避免立即查询到旧索引。
- `mail:sync-progress(status="completed", phase="poll")` 是同步心跳，不是数据变化事件；不得借它刷新 `["shell"]`、`["messages"]` 或 `["threads"]`。

### 4. Validation & Error Matrix
- 规则把新邮件归档/移动 -> `mail:new.folder_ids` 必须包含归档/目标文件夹，不能仍是原始 inbox。
- 规则删除或移除所有文件夹 -> 发送 `folder_ids=[]` 事件，并删除搜索索引中的旧文档。
- 新邮件进入 SQLite 但当前收件箱不刷新 -> 检查是否只失效了 `["messages", account_id]`。
- 批量同步 50 封新邮件 -> 前端 500ms 合并窗口内只触发一次列表/线程/shell 失效，避免每封邮件一轮 `/api/shell` + `/api/inbox`。
- 周期性 poll completed 没有数据变化 -> 只更新状态栏同步状态，不发起列表或 shell refetch。
- 切换账号/文件夹后邮件才出现 -> 说明换 key 触发了重新拉取，实时失效范围不足。
- 正文搜索第一次无结果、稍后切换视图才命中 -> 检查搜索缓存是否在索引提交后再次失效。

### 5. Good/Base/Bad Cases
- Good: 多账号“全部邮箱”模式下，任一账号收到 `mail:new` 都会失效 `["messages"]` 和 `["threads"]`，当前聚合收件箱自动重拉。
- Base: 单账号视图收到其他账号事件时，全局列表前缀失效可接受；它保证切换过去不会看到 60 秒内的旧缓存。
- Bad: 用 `account_id` 拼消息列表缓存 key，会漏掉以 `folderId` 为第二段的真实缓存。
- Bad: 在规则处理前发 `mail:new`，用户会先看到 inbox 新邮件，随后规则又把它移走，造成闪烁和重复请求。

### 6. Tests Required
- Rust 测试：规则把消息移动/归档后，`mail:new.folder_ids` 使用规则后的最终 folder ids。
- 前端测试：模拟 `mail:new`，断言调用 `invalidateQueries({ queryKey: ["messages"] })` 和 `["threads"]`。
- 前端测试：同一事件下断言文件夹/未读数按 `account_id` 精准失效。
- 前端测试：快速连续触发多个 `mail:new` 时，合并窗口内 messages/threads/shell 只刷新一次。
- 前端测试：搜索缓存在索引提交窗口后延迟失效。

### 7. Wrong vs Correct

#### Wrong
```typescript
queryClient.invalidateQueries({ queryKey: ["messages", accountId] });
queryClient.invalidateQueries({ queryKey: ["threads", accountId] });
```

#### Correct
```typescript
queryClient.invalidateQueries({ queryKey: ["messages"] });
queryClient.invalidateQueries({ queryKey: ["threads"] });
queryClient.invalidateQueries({ queryKey: ["folders", accountId] });
queryClient.invalidateQueries({ queryKey: ["folder-unread-counts", accountId] });
```

#### Wrong
```rust
// 规则处理前就把 provider 原始文件夹推给前端。
state.emit("mail:new", new_mail_event_payload(&stored).await);
```

#### Correct
```rust
// 规则处理和消息重载完成后，用最终 folder_ids 推送。
state.emit(crate::events::MAIL_NEW, new_mail_event_payload(state, &stored, &message, &folder_ids).await);
```

## Scenario: SSE 重连后的邮件 catch-up

### 1. Scope / Trigger
- Trigger: 浏览器到 `/events` 的 SSE 连接断开期间可能丢失 `mail:new`、pending ops 或同步完成事件；重连成功后必须补一次状态。
- 范围：`src/lib/sse-client.ts`、`src/app/useSseReconnectCatchup.ts`、`src/app/Layout.tsx`、`src/lib/api.ts`、`server/src/rpc/sync_cmd.rs`。

### 2. Signatures
- 前端重连订阅：`onSseReconnect(handler: () => void): () => void`。
- Layout hook：`useSseReconnectCatchup()`，在 authenticated layout 中注册一次。
- catch-up 后端入口：`wakeSync({ reason: "network_online", ensureRunning: true, pollIntervalSecs })` -> `POST /api/sync/wake`。
- 必刷缓存键：`["shell"]`、`["messages"]`、`["threads"]`。

### 3. Contracts
- 首次 SSE `open` 不触发 catch-up；只有“曾经连接成功 -> 断开 -> 重新连接成功”才调用 `onSseReconnect` handlers。
- 重连 catch-up 必须先 invalidate `["shell"]`、`["messages"]`、`["threads"]`，弥补断线期间丢失的事件。
- 非 Manual only 模式下，重连 catch-up 必须调用 `wakeSync(reason="network_online", ensureRunning=true, pollIntervalSecs=<当前设置>)`，让后端启动缺失 worker 或唤醒已有 worker。
- Manual only 模式下，重连 catch-up 只刷新缓存，不调用 `wakeSync`；用户仍需手动同步。
- `wakeSync` 失败属于后台补偿失败，调用点可以 `.catch(() => {})`，不要阻塞页面恢复。

### 4. Validation & Error Matrix
- 首次进入页面且 SSE 首次 open -> 不 invalidate、不 wake。
- SSE 已 open 后网络断开并重连成功 -> invalidate 三类缓存，并在非 manual 模式下 wake。
- Manual only + SSE 重连 -> invalidate 缓存，但不得启动后台同步。
- 重连时 `/api/sync/wake` 返回失败 -> 页面不崩溃，后续用户手动同步仍可重试。

### 5. Good/Base/Bad Cases
- Good: 笔记本睡眠后恢复，SSE 重连成功，前端立即重拉 shell/messages/threads，并唤醒增量同步追回期间变化。
- Base: 用户设置 Manual only，恢复网络后前端只丢弃陈旧缓存，不偷偷联网同步。
- Bad: 只依赖 EventSource 自动重连但不做 catch-up；断线期间的新邮件事件丢失后，列表要等下一次用户操作或定时刷新才变新。

### 6. Tests Required
- 前端单元测试：`onSseReconnect` 首次 open 不触发，断线重连成功才触发。
- 前端 hook 测试：重连后 invalidate `["shell"]`、`["messages"]`、`["threads"]`，并调用 `wakeSync(network_online, ensureRunning=true)`。
- 前端 hook 测试：Manual only 模式重连只 invalidate，不调用 `wakeSync`。

### 7. Wrong vs Correct

#### Wrong
```typescript
source.onopen = () => {
  queryClient.invalidateQueries({ queryKey: ["messages"] });
};
```

#### Correct
```typescript
onSseReconnect(() => {
  queryClient.invalidateQueries({ queryKey: ["shell"] });
  queryClient.invalidateQueries({ queryKey: ["messages"] });
  queryClient.invalidateQueries({ queryKey: ["threads"] });
  wakeSync({ reason: "network_online", ensureRunning: true, pollIntervalSecs });
});
```

## Scenario: 多账号同步唤醒入口

### 1. Scope / Trigger
- Trigger: Webmail 有多个邮箱账号时，窗口 focus、blur、网络恢复和手动同步都会触发同步；前端不能为每个账号串行调用 `startSync` 再 `triggerSync`。
- 范围：`server/src/api/sync.rs`、`server/src/rpc/sync_cmd.rs`、`src/lib/api-client.ts`、`src/lib/api.ts`、`src/app/useRealtimeSyncTriggers.ts`、`src/hooks/mutations/useSyncMutation.ts`。

### 2. Signatures
- 聚合入口：`POST /api/sync/wake`。
- 请求 body：
  - `account_ids?: string[]`，省略表示全部账号；显式空数组表示不唤醒任何账号。
  - `reason: string`，复用 `SyncTrigger::from_reason`：`manual`、`window_focus`、`window_blur`、`network_online`、`provider_push`、`startup`、`timer` 等。
  - `ensure_running?: boolean`，为 `true` 时先确保账号 worker 已启动。
  - `poll_interval_secs?: number`，仅在 `ensure_running=true` 时作为启动配置传入。
- 响应 body：`SyncWakeResult`，字段为 `account_count`、`ensured_count`、`triggered_count`、`one_shot_count`、`skipped_count`、`failures[]`。
- 旧底层入口保留：`POST /api/accounts/:id/sync/start`、`POST /api/accounts/:id/sync/trigger`、`POST /api/accounts/:id/sync/stop`。

### 3. Contracts
- 每个账号仍保持独立 sync worker、stop channel、trigger channel、backoff 和 provider 状态；不要合并成全局 worker。
- 前端被动实时事件（窗口 focus、blur、网络从 offline 恢复 online）必须调用一次 `wakeSync({ accountIds, reason, ... })`，不要按账号循环 `startSync + triggerSync`。
- `window_focus` 和 `network_online` 使用 `ensureRunning=true`，并传当前 `pollIntervalSecs`。
- `/api/sync/wake` 在 `ensureRunning=true` 且某账号 worker 是本次刚启动时，不得再给同一账号立刻投递同一个 trigger；启动后的首轮同步已经覆盖这次唤醒。
- `/api/sync/wake` 在 `ensureRunning=true` 但 worker 已存在时，仍必须投递 trigger，用于打断等待、更新运行态或刷新 backoff。
- `window_blur` 使用 `ensureRunning=false`，只通知已存在 worker 更新运行态；缺失 worker 时不得启动一轮同步。
- `manual` 手动同步使用 `ensureRunning=false`；若 worker 不存在，后端允许启动一次 `poll_interval_secs=0` 的 one-shot 同步。
- 旧底层入口 `POST /api/accounts/:id/sync/trigger` 只能在 `SyncTrigger::should_sync_now()` 为 true 时启动缺失 worker；`manual`/`provider_push` 可以 one-shot，`window_blur`/`timer` 等 passive reason 不得启动。
- `manual` 偏好下，focus/blur/network 这类被动事件不得调用 `wakeSync`；只有用户点击“立即同步”或新增账号初始同步才允许 one-shot。
- `src/lib/api.ts` 的 `wakeSync` 必须在 `failures.length > 0` 时抛出错误，让手动同步和新增账号能看到失败；忽略型被动事件可在调用点 `.catch(() => {})`。

### 4. Validation & Error Matrix
- `account_ids` 中含空字符串 -> `400`/validation error。
- `account_ids` 重复 -> 后端按出现顺序去重。
- `account_ids` 省略且当前无账号 -> `200`，`account_count=0`。
- `ensure_running=true` 且某账号启动失败 -> 响应 `failures[]` 记录该账号；其他账号继续处理。
- `ensure_running=true` 且 worker 本次刚启动 -> `ensured_count` 增加，`triggered_count` 不因同一 trigger 增加。
- `ensure_running=true` 且 worker 已存在 -> `ensured_count` 增加，并继续投递 trigger，`triggered_count` 增加。
- `ensure_running=false` + `window_blur` + worker 不存在 -> 不启动 worker，`skipped_count` 增加。
- `ensure_running=false` + `manual` + worker 不存在 -> 启动 one-shot，`one_shot_count` 增加。
- 旧 `/api/accounts/:id/sync/trigger` + `window_blur` + worker 不存在 -> 不启动 one-shot。

### 5. Good/Base/Bad Cases
- Good: 用户回到页面时，前端只发一次 `/api/sync/wake`，后端为所有账号确保 worker 运行并发送 `window_focus` trigger。
- Good: 应用冷启动后 `network_online` wake 新建 worker，只跑 worker 启动自带的首轮同步，不再紧接着重复 poll 一次。
- Base: 用户在 Manual only 模式切回页面，不发请求；点击状态栏同步按钮时，单账号通过 `manual` wake 运行一轮。
- Bad: 前端对 5 个账号分别调用 `/sync/start` 和 `/sync/trigger`，会产生 10 个请求，并把 worker 生命周期细节泄漏到 UI。
- Bad: `window_blur` 在没有 worker 时启动 one-shot，会让用户只是切走页面也触发同步。
- Bad: `ensureRunning=true` 新建 worker 后仍立即投递 `window_focus`，会形成“启动首轮 poll + trigger poll”双拉取。

### 6. Tests Required
- 前端 hook 测试：focus/network online 只调用一次 `wakeSync`，body 包含全部账号、`ensureRunning=true` 和 `pollIntervalSecs`。
- 前端 hook 测试：blur 只调用一次 `wakeSync`，`ensureRunning=false`。
- 前端 hook 测试：Manual only 下被动 focus 不调用 `wakeSync`。
- 前端 API 测试：`wakeSync` 序列化为 `/api/sync/wake` 和 snake_case body；`failures[]` 非空时抛错。
- Rust 服务测试：`account_ids` 去重并拒绝空 ID；passive trigger 不启动缺失 worker，manual trigger 可 one-shot。
- Rust 服务测试：`wakeSync(ensureRunning=true)` 新启动 worker 时不重复 trigger，worker 已存在时仍 trigger。
- Rust 服务测试：旧 trigger 入口对 passive reason 不启动缺失 worker，manual/provider_push 仍可启动。
- Rust/OpenAPI 检查：`/api/sync/wake` 在 docs 中登记，并且 `/api/*` 仍走 session 鉴权。

### 7. Wrong vs Correct

#### Wrong
```typescript
for (const accountId of accountIds) {
  await startSync(accountId, pollInterval);
  await triggerSync(accountId, "window_focus");
}
```

#### Correct
```typescript
await wakeSync({
  accountIds,
  reason: "window_focus",
  ensureRunning: true,
  pollIntervalSecs: pollInterval,
});
```

#### Wrong
```rust
// ensureRunning 已经启动 worker 后，又立刻 dispatch 同一个 trigger。
start_sync_inner(state.clone(), account_id.clone(), poll_interval).await?;
dispatch_sync_trigger(state, account_id, trigger, true).await?;
```

#### Correct
```rust
let outcome = start_sync_inner(state.clone(), account_id.clone(), poll_interval).await?;
if outcome == SyncStartOutcome::AlreadyRunning {
    dispatch_sync_trigger(state, account_id, trigger, false).await?;
}
```

## Scenario: Webmail 启动快照与元数据缓存

### 1. Scope / Trigger
- Trigger: 多账号 Webmail 首屏和设置页需要 accounts、folders、未读数、Gmail realtime 配置；前端不得因账号数量线性发起 N+1 元数据请求。
- 范围：`server/src/api/shell.rs`、`src/lib/api-client.ts`、`src/lib/api.ts`、`src/hooks/queries/useShellQuery.ts`、`src/hooks/queries/useAccountsQuery.ts`、`src/hooks/queries/useFoldersQuery.ts`、`src/hooks/queries/useFolderUnreadCounts.ts`、`src/features/settings/AccountsTab.tsx`、`src/components/StatusBar.tsx`。

### 2. Signatures
- 启动快照：`GET /api/shell`。
- 响应 body：
  - `accounts: Account[]`
  - `folders: Record<accountId, Folder[]>`
  - `unreadCounts: Record<accountId, Record<folderId, number>>`
  - `gmailRealtime: Record<accountId, GmailRealtimeConfig>`
- 前端 shell cache key：`["shell"]`。
- 派生缓存 key：
  - `["accounts"]`
  - `["folders", accountId]`
  - `["folder-unread-counts", accountId]`
  - `["gmail-realtime", accountId]`

### 3. Contracts
- `GET /api/shell` 是账号元数据首次加载的权威入口；前端元数据 hooks 应通过 shell 快照填充派生 React Query 缓存。
- `accounts`、`folders`、`unreadCounts`、`gmailRealtime` 必须在一次 shell 响应中返回；缺少某类数据时返回空数组/空对象，不省略字段。
- `gmailRealtime` 只包含 Gmail 账号；非 Gmail 账号不得返回伪配置。
- 底层单资源接口仍保留作局部 fallback：`GET /api/accounts`、`GET /api/accounts/:id/folders`、`GET /api/accounts/:id/gmail-realtime`。
- `fetchShellSnapshot(queryClient)` 必须通过 `["shell"]` 去重并 hydrate 派生缓存，避免同一首屏中多个 hooks 并发打出多次 `/api/shell`。
- `StatusBar.refreshMailQueries()` 和网络恢复流程必须在“确有变化”的事件上 invalidate `["shell"]`，再刷新 messages/threads 等视图缓存，避免 shell 派生元数据长期陈旧。
- `mail:sync-progress(status="completed", phase="poll")` 是周期性同步心跳，不代表数据变化；前端只更新同步状态，不得 invalidate `["shell"]`、`["messages"]` 或 `["threads"]`，否则会把低延迟轮询变成每几秒全量重拉。
- 首屏不拉取全部历史邮件或正文；messages/threads 仍按当前文件夹分页获取。

### 4. Validation & Error Matrix
- 无账号 -> `accounts=[]`、`folders={}`、`unreadCounts={}`、`gmailRealtime={}`。
- 某账号 folders/unreadCounts 加载失败 -> shell 对该账号返回空集合并继续返回其他账号数据。
- 某 Gmail realtime 配置加载失败 -> 记录 warning，`gmailRealtime` 跳过该账号，shell 仍成功。
- SSE `mail:new` / pending ops changed / `mail:sync-complete` worker 退出 / 非 `poll` 阶段完成 -> invalidate `["shell"]`、`["messages"]`、`["threads"]`，并按账号精准刷新 folders/unreadCounts 派生 key。
- SSE `mail:sync-progress(status="completed", phase="poll")` -> 只把状态置为 idle，不发起 shell/messages/threads refetch。
- 网络从 offline 恢复 online -> `wakeSync(reason="network_online", ensureRunning=true)` 后 invalidate `["shell"]`、`["messages"]`、`["threads"]`。

### 5. Good/Base/Bad Cases
- Good: 两个账号进入收件箱时，`useAccountsQuery()` 和 `useFoldersForAccountsQuery(["a","b"])` 只产生一次 `/api/shell` 网络请求，并填充账号与文件夹缓存。
- Base: 进入 Settings → Accounts 时，账号行直接使用 shell 中的 Gmail realtime 配置展示状态。
- Bad: `AccountsTab` 对每个 Gmail 账号循环调用 `getGmailRealtimeConfig(accountId)`，账号越多请求越多。
- Bad: 收到新邮件后只 invalidate `["folders", accountId]`，但 shell cache 仍新鲜，下一次派生查询继续读旧 shell。

### 6. Tests Required
- Rust API 测试：`/api/shell` 响应必须包含 `accounts`、`folders`、`unreadCounts`、`gmailRealtime`。
- 前端 hook 测试：`useAccountsQuery` + `useFoldersForAccountsQuery` 并发时只请求一次 `/api/shell`，并返回派生数据。
- 前端设置页测试：账号行使用 shell 中的 Gmail realtime 配置，不调用 N 次 `getGmailRealtimeConfig`。
- 前端网络恢复测试：offline→online 时调用 `wakeSync`，并 invalidate `["shell"]`、`["messages"]`、`["threads"]`。
- 前端构建测试：`ShellData`、`GmailRealtimeConfig` 类型通过 `tsc`。

### 7. Wrong vs Correct

#### Wrong
```typescript
const accounts = await listAccounts();
for (const account of accounts) {
  await listFolders(account.id);
  if (account.provider === "gmail") {
    await getGmailRealtimeConfig(account.id);
  }
}
```

#### Correct
```typescript
const shell = await fetchShellSnapshot(queryClient);
const folders = shell.folders[accountId] ?? [];
const gmailRealtime = shell.gmailRealtime[accountId];
```

## Scenario: Gmail History 增量游标可靠性

### 1. Scope / Trigger
- Trigger: Gmail 增量同步依赖 History API；HTTP 错误、分页遗漏或过早推进 cursor 都会导致邮件永久漏同步。
- 范围：`crates/pebble-mail/src/gmail_sync.rs`、`crates/pebble-mail/src/provider/gmail.rs`、`pebble-store` 中账号同步状态持久化。

### 2. Signatures
- Gmail History 请求：`GET https://www.googleapis.com/gmail/v1/users/me/history?startHistoryId=<history_id>[&pageToken=<token>]`。
- 响应字段：`history[]`、`nextPageToken?: string`、`historyId?: string`。
- 本地收集结果：`new_ids`、`deleted_ids`、`labels_added`、`labels_removed`、`history_id`。
- 错误映射：`401 -> PebbleError::Auth`，`404 -> PebbleError::Network("Gmail history expired; full resync required ...")`，其他非 2xx -> `PebbleError::Network`。

### 3. Contracts
- 调用 `resp.json()` 前必须先检查 HTTP status；非 2xx 不得当作空 history 静默成功。
- `nextPageToken` 非空时必须继续请求下一页，并把所有页的 added/deleted/label changes 合并后再进入本地处理。
- `historyId` cursor 只能在所有分页拉取成功、所有本地 message/label/delete 处理完成且 `failure_count == 0` 后推进。
- `401` 表示授权失效，必须作为 Auth 错误暴露给上层，不得重试成普通网络抖动。
- `404` 表示 Gmail history cursor 过期；当前契约是显式返回“需要 full resync”的 Network 错误，不能推进旧 cursor，也不能吞掉错误。
- 单页无 `history` 但有 `historyId` 是合法空增量；仍按 cursor 推进规则处理。

### 4. Validation & Error Matrix
- History response `200` + `nextPageToken="p2"` -> 必须继续请求 `pageToken=p2`。
- 第二页处理失败 -> 不推进 cursor，下一轮从旧 `startHistoryId` 重试。
- History response `401` -> 返回 `PebbleError::Auth`，等待重新授权。
- History response `404` -> 返回包含 `full resync required` 的 Network 错误，保留旧 cursor。
- History response `500` 或非法 JSON -> 返回 Network 错误，保留旧 cursor。
- 所有分页成功但某封新邮件 fetch/store 失败 -> `failure_count > 0`，不推进 cursor。

### 5. Good/Base/Bad Cases
- Good: Gmail 一次返回两页 history，Pebble 合并两页的新增、删除和 label 变化，再统一处理并在全部成功后推进到最新 `historyId`。
- Base: 没有变化时 Gmail 返回空 history 和新 `historyId`，Pebble 可以推进 cursor，避免重复查询同一区间。
- Bad: 忽略 `nextPageToken` 只处理第一页，第二页新邮件永远不会进入本地库。
- Bad: `404` 时仍把响应解析成空 history 并推进 cursor，会把过期区间内的所有变化永久跳过。

### 6. Tests Required
- Rust 单元测试：分页收集函数跟随 `nextPageToken`，合并新增、删除、label added/removed。
- Rust 单元测试：`401` 映射为 Auth，`404` 错误消息保留“full resync required”。
- Rust 同步测试：本地处理有失败时 `can_advance_gmail_cursor(false)` 路径不推进 cursor。
- 回归测试：非法/非 2xx History response 不会被当成空增量成功。

### 7. Wrong vs Correct

#### Wrong
```rust
let history: HistoryList = resp.json().await?;
advance_cursor(history.history_id);
```

#### Correct
```rust
let history = collect_paginated_gmail_history(&history_id, fetch_page).await?;
if failure_count == 0 {
    advance_cursor(history.history_id);
}
```

## Scenario: 浏览器 Web Push 通知契约

### 1. Scope / Trigger
- Trigger: Pebble 支持页面关闭后的浏览器系统通知，链路横跨 Service Worker、前端设置页、REST API、SQLite 订阅存储、规则后新邮件处理和 VAPID 配置。
- 范围：`public/pebble-sw.js`、`src/lib/web-push.ts`、`src/features/settings/GeneralTab.tsx`、`server/src/api/notifications.rs`、`server/src/push_notifications.rs`、`crates/pebble-store/src/notification_devices.rs`。

### 2. Signatures
- `GET /api/notifications/vapid-public-key` -> `{ public_key: string }`。
- `GET /api/notifications/devices` -> `{ devices: NotificationDevice[] }`。
- `POST /api/notifications/subscriptions` body `{ device_id: string, device_name?: string, subscription: { endpoint: string, keys: { p256dh: string, auth: string } } }` -> `{ device: NotificationDevice }`。
- `DELETE /api/notifications/subscriptions/:device_id` -> `null`。
- `PATCH /api/notifications/devices/:device_id` body `{ device_name: string }` -> `NotificationDevice`。
- `DELETE /api/notifications/devices/:device_id` -> `null`。
- `POST /api/notifications/test` body `{ device_id: string }` -> `null`。
- DB table: `notification_devices(id, endpoint, p256dh, auth, device_name, user_agent, status, session_id, session_expires_at, last_active_at, summary_sent_at, created_at, updated_at)`。

### 3. Contracts
- 所有 `/api/notifications/*` 路由必须经过 cookie session 鉴权；不要加入 auth exempt 白名单。
- 前端必须只在用户点击通知开关时调用 `Notification.requestPermission()`；自动恢复只能在 `Notification.permission === "granted"` 时重新登记订阅。
- 新设备默认通知关闭；本地 `pebble-notifications-enabled=true` 只表示旧偏好，可在权限已允许时恢复，否则必须写回关闭。
- Manual only 模式不得开启通知；切到 Manual only 时当前设备订阅要取消。
- `PEBBLE_VAPID_PRIVATE_KEY` 是可选 base64url 私钥；缺省时自动生成并保存到 `secure_user_data` 的 `web_push_vapid_private_key`。
- `PEBBLE_VAPID_PUBLIC_KEY` 可选；若设置，必须和私钥推导出的公钥一致，否则启动失败。
- 规则处理后最终仍在收件箱、未读、未删除且 `StoredMessage.notify=true` 的新邮件才进入 Web Push 队列。
- 普通邮件 5 秒合并；验证码/OTP 邮件立即推送；普通邮件 `allowForeground=false`，测试和摘要 `allowForeground=true`。
- OTP 判断和验证码展示是两层逻辑：强关键词（如 `verification code`、`OTP`、`验证码`、`one-time`）可直接触发 OTP 即时推送；弱关键词（如单独的 `code`、`verify`、`verification`）必须同时出现 4-8 位含数字 token 才触发。
- OTP payload 即使没有可展示验证码也必须保持 `kind="otp"` 和 `allowForeground=true`；验证码展示只使用可信提取结果，优先关键词附近 token，再全文兜底，并保留原文大小写。
- OTP code 全文兜底时要过滤明显年份和日期 token（如 `2026`、`0527`、`20260527`），避免把过期时间误展示成验证码。
- 新邮件通知去重是内存级近期窗口：同一 `message_id` 在 1 小时内只通知一次，最多保留 4096 条，超出时清理最旧记录。
- 单封 payload 带 `messageId`，点击后前端打开收件箱中的该邮件并标已读；批量/摘要 payload 不带 `messageId`，点击只打开收件箱。
- 服务端启动后必须暂停已有设备，避免内存 session 丢失后继续向旧 session 发送通知；重新登录且浏览器权限仍允许时由前端恢复订阅。

### 4. Validation & Error Matrix
- `device_id` 为空 -> `400`。
- `subscription.endpoint/p256dh/auth` 任一为空 -> `400`。
- 重命名 `device_name` 为空 -> `400`。
- 测试通知设备不存在 -> API 错误响应，不应静默成功。
- `PEBBLE_VAPID_PRIVATE_KEY` 为空或格式非法 -> 进程启动失败。
- `PEBBLE_VAPID_PUBLIC_KEY` 为空或和私钥不匹配 -> 进程启动失败。
- Push 服务返回永久端点错误 -> 删除该通知设备，避免持续重试坏订阅。

### 5. Good/Base/Bad Cases
- Good: 用户在 Settings → General 点击开启，浏览器权限弹窗出现，后端保存当前设备订阅；页面关闭后新收件箱未读邮件触发系统通知。
- Base: 旧本地偏好为 true 但浏览器权限还不是 granted，页面加载时不弹权限请求，状态回到关闭，等待用户手动点击。
- Bad: 在 `mail:new` SSE 发出前或规则执行前推送通知，会把已经被规则归档/移动的邮件误通知给用户。
- Bad: 只设置 `PEBBLE_VAPID_PUBLIC_KEY` 但没有匹配私钥，会导致订阅看似成功、实际推送全部失败。

### 6. Tests Required
- Rust store 测试：设备 upsert/暂停、未读收件箱摘要只统计 inbox 未读未删除邮件。
- Rust push 测试：强 OTP 关键词无 code 也触发即时推送但不展示 code；弱关键词必须搭配 code-like token；提取出的字母数字验证码必须保留原文大小写；年份/日期 token 不得抢占真正 code；通知去重必须覆盖 1 小时 TTL 和 4096 容量兜底。
- Rust migration 测试：旧版本迁移到 `CURRENT_VERSION` 后 `notification_devices` 表存在。
- 前端 store 测试：新设备通知默认关闭；通知点击遇到 dirty compose 时会保留草稿保护，确认后再打开 pending 邮件，取消后清空 pending 邮件。
- 前端构建测试：`src/lib/web-push.ts` 的 `applicationServerKey` 类型必须通过 `tsc`。
- 手动浏览器验证：HTTPS/localhost 下测试通知、普通通知前台抑制、单封点击标已读、批量点击只打开收件箱。

### 7. Wrong vs Correct

#### Wrong
```typescript
// 页面加载时主动请求权限，会被浏览器拦截，也违背“用户点击才请求”。
await Notification.requestPermission();
```

#### Correct
```typescript
// 只有用户点击 Settings 开关后才请求权限。
await enableCurrentDeviceNotifications();
```

#### Wrong
```rust
// 规则执行前就通知，可能误报已经被规则归档的邮件。
state.push_notifications.queue_mail(store, candidate).await;
```

#### Correct
```rust
// 规则处理后，用最终 folder_ids 判断是否仍在 inbox。
notify_new_message_after_rules(state, store, &message, &folder_ids, stored.notify, deferred).await;
```
