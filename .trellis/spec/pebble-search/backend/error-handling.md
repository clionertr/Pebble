# 错误处理

> pebble-search 的错误分类和传播约定。

---

## 总览

库层统一返回 `pebble_core::Result<T>` 或显式 `PebbleError`。HTTP 状态码映射只在 `server/src/api/error.rs` 完成。

---

## 错误类型

- Tantivy/IO 错误映射 PebbleError，调用方决定是否重试或重建索引。
- 搜索 query 解析失败应返回可诊断错误，不 panic。
- 删除/清空索引失败不能静默吞掉，除测试清理外要记录或传播。

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
