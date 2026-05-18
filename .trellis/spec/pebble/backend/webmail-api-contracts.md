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
