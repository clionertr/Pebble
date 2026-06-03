# 目录结构

> pebble-translate 后端代码组织约定。

---

## 总览

`pebble-translate` 负责DeepL、DeepLX、Generic API、LLM 翻译请求和响应解析。它位于 `crates/pebble-translate/src/`，只暴露 crate API，不定义 HTTP 路由。

---

## 目录布局

```text
crates/pebble-translate/src/
├── types.rs
├── deepl.rs
├── deeplx.rs
├── generic.rs
├── llm.rs
└── lib.rs
```

---

## 模块组织

- `types.rs`：翻译 provider 配置和结果类型。
- `deepl.rs`：DeepL 请求格式。
- `deeplx.rs`：DeepLX 请求/响应。
- `generic.rs`：JSON path 解析。
- `llm.rs`：LLM/stream 翻译。
- `lib.rs`：TranslateService。

新增能力优先放入职责最接近的现有模块；只有当现有模块已经承载多个独立状态机或协议边界时才新增文件。

---

## 命名约定

- 文件名使用 snake_case。
- 公共类型名描述领域对象或协议对象，不使用 HTTP/Tauri/RPC 命名。
- 只在 crate 外确实需要时使用 `pub`；crate 内共享优先 `pub(crate)`。

---

## 示例

以当前 `src/lib.rs` 的公共导出和上方模块职责为准；新增模块后同步更新本文件和 `index.md`。
