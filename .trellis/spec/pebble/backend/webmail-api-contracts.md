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
- `mail:new.account_id` 是“事件来源账号”，不是消息/线程查询的 React Query key。
- 收到 `mail:new` 后，前端必须用 `["messages"]` 和 `["threads"]` 前缀失效列表缓存；不能用 `["messages", account_id]` 或 `["threads", account_id]`。
- 文件夹和未读数仍按账号精准失效：`["folders", account_id]`、`["folder-unread-counts", account_id]`。
- 搜索索引由后端批量提交；前端收到 `mail:new` 后应延迟一次 `["search"]` 失效，避免立即查询到旧索引。

### 4. Validation & Error Matrix
- 新邮件进入 SQLite 但当前收件箱不刷新 -> 检查是否只失效了 `["messages", account_id]`。
- 切换账号/文件夹后邮件才出现 -> 说明换 key 触发了重新拉取，实时失效范围不足。
- 正文搜索第一次无结果、稍后切换视图才命中 -> 检查搜索缓存是否在索引提交后再次失效。

### 5. Good/Base/Bad Cases
- Good: 多账号“全部邮箱”模式下，任一账号收到 `mail:new` 都会失效 `["messages"]` 和 `["threads"]`，当前聚合收件箱自动重拉。
- Base: 单账号视图收到其他账号事件时，全局列表前缀失效可接受；它保证切换过去不会看到 60 秒内的旧缓存。
- Bad: 用 `account_id` 拼消息列表缓存 key，会漏掉以 `folderId` 为第二段的真实缓存。

### 6. Tests Required
- 前端测试：模拟 `mail:new`，断言调用 `invalidateQueries({ queryKey: ["messages"] })` 和 `["threads"]`。
- 前端测试：同一事件下断言文件夹/未读数按 `account_id` 精准失效。
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
