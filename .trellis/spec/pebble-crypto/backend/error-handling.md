# 错误处理

> pebble-crypto 的错误分类和传播约定。

---

## 总览

库层统一返回 `pebble_core::Result<T>` 或显式 `PebbleError`。HTTP 状态码映射只在 `server/src/api/error.rs` 完成。

---

## 错误类型

- 解密失败返回 PebbleError，调用方决定是否暴露为重新授权/内部错误。
- 密钥文件缺失可生成；格式错误必须失败，不能静默重置导致数据不可解。
- 错误信息不得包含密钥内容或明文片段。

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
