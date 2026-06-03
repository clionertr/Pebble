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

## API/RPC/store 边界

第三阶段架构清债以边界职责为准，而不是简单按“函数薄不薄”删除层级。可以把三层理解成：

- **API 层是柜台**：只接收 HTTP 输入，做路径、查询、JSON、Multipart、Cookie/Header 的提取和基础格式校验，把业务请求交给服务层；找不到资源、认证失败等 HTTP 语义在这里映射成状态码。
- **RPC/service 层是后厨**：承载跨 store、搜索、推送、OAuth、同步器的业务编排。例如通知订阅注册需要保存设备、推断设备名、发送首次未读摘要，这些不应留在 handler 里。
- **store/crates 是账本和领域能力**：`pebble-store` 负责持久化查询和事务，`pebble-search` 负责索引检索，`pebble-mail` 负责协议和同步，避免反向依赖 `server/src/api/`。

### 允许保留的薄服务函数

薄函数不一定是坏味道。满足任一条件时可以保留：

- 它定义了稳定的 API → service 边界，后续可能增加校验、日志、缓存、事件或多 store 编排。
- 它统一隐藏历史命名（`rpc` 目录当前作为内部 service 层，不再代表 JSON-RPC）。
- 它避免 API handler 直接依赖 store 细节，让 handler 保持可扫描、可替换。

### 应该收敛或删除的薄函数

满足以下条件时优先改 `pub(crate)`、合并或删除：

- 只有单个调用方，且函数完全等价于一行 store 调用，并且没有业务边界价值。
- 对外 `pub` 但只在本 crate 内使用。
- 名称仍暗示 Tauri/IPC/RPC 命令，实际已没有对应入口。

### 阻塞 I/O 约定

- API handler 不直接执行可能阻塞 Tokio runtime 的 SQLite 或文件 I/O。
- 需要调用现有 `Store` 方法时，优先用 `Store::with_blocking_async`，避免在每个 service 函数重复写 `tokio::task::spawn_blocking` 和 join-error 转换。
- 新增 store 查询时，优先在 `pebble-store` 内通过 `with_read` / `with_write` 封装；异步调用方用 `with_read_async` / `with_write_async`。

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
