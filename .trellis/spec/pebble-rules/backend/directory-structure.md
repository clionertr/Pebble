# 目录结构

> pebble-rules 后端代码组织约定。

---

## 总览

`pebble-rules` 负责规则类型、条件匹配和动作评估。它位于 `crates/pebble-rules/src/`，只暴露 crate API，不定义 HTTP 路由。

---

## 目录布局

```text
crates/pebble-rules/src/
├── types.rs
├── matcher.rs
└── lib.rs
```

---

## 模块组织

- `types.rs`：规则条件/动作结构。
- `matcher.rs`：条件匹配器。
- `lib.rs`：RuleEngine 和公共 API。

新增能力优先放入职责最接近的现有模块；只有当现有模块已经承载多个独立状态机或协议边界时才新增文件。

---

## 命名约定

- 文件名使用 snake_case。
- 公共类型名描述领域对象或协议对象，不使用 HTTP/Tauri/RPC 命名。
- 只在 crate 外确实需要时使用 `pub`；crate 内共享优先 `pub(crate)`。

---

## 示例

以当前 `src/lib.rs` 的公共导出和上方模块职责为准；新增模块后同步更新本文件和 `index.md`。
