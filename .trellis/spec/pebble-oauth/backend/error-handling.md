# 错误处理

> pebble-oauth 的错误分类和传播约定。

---

## 总览

库层统一返回 `pebble_core::Result<T>` 或显式 `PebbleError`。HTTP 状态码映射只在 `server/src/api/error.rs` 完成。

---

## 错误类型

- 授权/刷新失败返回 PebbleError::OAuth 或 Auth 语义，由 server 层转成用户可读提示。
- 错误消息可包含 provider 和 HTTP 状态，但不得包含 code、token、secret。
- URL 解析和代理配置错误必须在启动/发起授权时立即失败。

---

## 传播模式

```rust
let value = fallible_call().map_err(|e| PebbleError::Network(format!("operation failed: {e}")))?;
```

保留操作名和原始错误；不要写 `map_err(|_| ...)` 丢失上下文，除非分支明确表示 timeout 且没有底层错误对象。

---

## 常见错误

- 把内部错误字符串直接交给 API 客户端。
- 在库层打印敏感数据。
- 把正常缺失和真正损坏混成同一种错误。
