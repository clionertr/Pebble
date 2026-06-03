# 错误处理

> pebble-mail 的错误分类和传播约定。

---

## 总览

库层统一返回 `pebble_core::Result<T>` 或显式 `PebbleError`。HTTP 状态码映射只在 `server/src/api/error.rs` 完成。

---

## 错误类型

- 认证失败映射 Auth，网络/协议失败映射 Network，解析和本地状态失败保留具体操作名。
- IMAP 命令使用 with_imap_timeout；SMTP/Gmail/Outlook HTTP 错误要先检查 status 再解析 body。
- 断开连接失败可记录 warning 或在恢复路径中忽略，但不能覆盖主操作结果。

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
