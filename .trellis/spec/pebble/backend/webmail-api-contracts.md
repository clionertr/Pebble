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
