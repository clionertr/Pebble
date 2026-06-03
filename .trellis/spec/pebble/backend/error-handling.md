# 错误处理规范

> 基于第 1 阶段安全止血总结出的项目实际约定。

---

## 错误类型体系

### ApiError（HTTP 层）

定义位置：`server/src/api/error.rs`

```rust
pub enum ApiError {
    BadRequest(String),
    Unauthorized(String),
    NotFound(String),
    Internal(String),
}
```

**核心理则**：`Internal` 变体必须使用**固定安全文案**，不得将内部错误详情直接暴露给客户端。

```rust
// 正确：内部错误对客户端脱敏
ApiError::internal("Internal server error")

// 错误：泄漏内部信息（已在 Phase 1 修复）
ApiError::internal(e.to_string())

// 权限错误带消息是允许的
ApiError::unauthorized("Invalid credentials")
```

### PebbleError（业务层）

定义位置：`crates/pebble-core/src/error.rs`

RPC 层返回 `Result<T, PebbleError>`。API handler 通过 `From<PebbleError>` 自动转换：
- 认证/验证类错误 → `ApiError::Unauthorized` / `ApiError::BadRequest`
- 其他所有 → `ApiError::Internal("Internal server error")`

**禁止**：handler 中直接 `ApiError::internal(e.to_string())`，除非错误消息本身不包含路径/数据库/网络细节。

---

## 错误处理模式

### 禁止 `.unwrap()` 在请求可达路径

**规则**：所有 HTTP 可达路径（API handler、RPC 函数被 handler 调用的分支）不得出现 `.unwrap()`。

**例外**：
- JSON 结构体字面构造中的 `serde_json::json!({})["paths"].as_object_mut().unwrap()` —— `json!` 宏保证字段存在
- 测试代码中 `.unwrap()` 是允许的

**已确认安全的位置**：`server/src/api/docs.rs:41` — `spec["paths"].as_object_mut().unwrap()`，因为上一行刚用 `json!` 创建了 `paths: {}`。

### 禁止 `let _ =` 吞没关键错误

**规则**：对可能导致数据不一致或状态丢失的失败操作，不得 `let _ =` 静默丢弃。

```rust
// 错误：账户回滚失败被忽略（Phase 1 已修复）
let _ = state.store.delete_account(&account.id);

// 正确：记录警告日志
if let Err(e) = state.store.delete_account(&account.id) {
    tracing::warn!("Failed to rollback account {}: {e}", account.id);
}
```

**允许 `let _ =` 的场景**：
- IMAP 连接主动断开（`imap.disconnect().await`）
- 临时文件/目录清理（`std::fs::remove_dir_all`）
- 事件广播失败（`tx.send(...)`）

### 错误传播标准模式

```rust
// API handler 中统一使用 ? 传播
async fn handler(
    State(state): State<Arc<AppState>>,
    ...
) -> Result<Json<T>, ApiError> {
    let result = rpc::do_stuff(State(state), ...).await?; // PebbleError → ApiError
    Ok(Json(result))
}
```

### 日志保留内部错误详情

当内部错误发生但需要对客户端脱敏时：

```rust
// 正确：日志记录详情，客户端得到安全消息
state.store.do_something().map_err(|e| {
    tracing::error!("Failed to do something: {e}");
    ApiError::internal("Internal server error")
})?;
```

---

## 常见错误

1. **`map_err(|_| ...)` 丢弃原始错误** — `crates/pebble-mail/src/imap.rs` 仍有 11 处，应在第 2 阶段改为保留原始错误或分类错误（区分超时/连接失败/TLS错误）
2. **错误返回类型不一致** — 已修复的历史问题：`auth_api.rs` 曾返回 `(StatusCode, Json<Value>)` 绕过 `ApiError`，`health.rs` 曾返回 `Result<..., String>` 而非 `PebbleError`。新增 HTTP handler 必须继续走 `ApiError`，RPC/service 层继续返回 `PebbleError`。

---

## 测试要求

- 每个 handler 的错误路径应至少有一个测试（如权限拒绝、资源不存在、参数无效）
- `ApiError::from(PebbleError)` 转换逻辑应有单元测试覆盖分类映射
