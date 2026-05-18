# 目录结构

> Pebble 后端代码的实际组织方式。

---

## 总览

Pebble 是 React SPA + Rust HTTP API + SSE 的自托管 Webmail。主后端位于 `server/`，可复用领域能力位于 `crates/`。

- **主服务**：`server/src/main.rs` 启动 Axum、挂载 REST API、SSE、OAuth 和 Gmail webhook。
- **API 层**：`server/src/api/` 只做 HTTP 提取、校验、响应包装，业务逻辑委托到 `server/src/rpc/` 或 `crates/`。
- **服务层**：`server/src/rpc/` 是历史命名，现作为内部服务层使用；不要再新增 JSON-RPC 分发入口。
- **领域 crate**：`crates/` 存放可测试、可复用的邮件、存储、搜索、加密、OAuth、规则、翻译和隐私处理逻辑。

---

## 目录布局

```text
Pebble/
├── src/                    # React 前端 SPA
│   ├── app/                # 应用壳、SSE/实时同步 hooks
│   ├── features/           # inbox、compose、settings、auth 等功能
│   ├── lib/                # REST API client、SSE client、API 类型
│   └── stores/             # Zustand 状态
├── server/                 # Rust HTTP 后端
│   └── src/
│       ├── main.rs         # 进程入口、路由注册、后台任务启动
│       ├── api/            # REST API handler
│       ├── middleware/     # Cookie session 鉴权
│       ├── session.rs      # 会话存储和登录限流
│       ├── gmail_realtime.rs
│       ├── snooze_watcher.rs
│       └── rpc/            # 内部服务层，不再暴露 /rpc
└── crates/                 # Rust workspace crates
    ├── pebble-core/
    ├── pebble-store/
    ├── pebble-mail/
    ├── pebble-search/
    ├── pebble-crypto/
    ├── pebble-oauth/
    ├── pebble-rules/
    ├── pebble-translate/
    └── pebble-privacy/
```

---

## 约定

- 新的浏览器可见能力必须通过 `server/src/api/` 暴露 REST 端点，不新增 `/rpc` 或 JSON-RPC 分发。
- API handler 保持薄层：解析 `Path/Query/Json/Multipart`，调用内部服务层，返回 `Json<T>` 或流式响应。
- 前端只通过 `src/lib/api-client.ts`、`src/lib/api.ts` 和 `src/lib/sse-client.ts` 访问后端。
- 共享 TypeScript 类型放在 `src/lib/api-types.ts`，命名必须描述 Web API 契约，不使用 IPC/Tauri 命名。

---

## 反例与正确做法

### 错误

```text
src-tauri/src/rpc/dispatch.rs
/rpc/batch
invoke("list_accounts")
```

### 正确

```text
server/src/api/accounts.rs
GET /api/accounts
apiGet<Account[]>("/api/accounts")
```
