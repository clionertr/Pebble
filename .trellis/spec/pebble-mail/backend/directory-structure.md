# 目录结构

> pebble-mail 后端代码组织约定。

---

## 总览

`pebble-mail` 负责IMAP/SMTP、Gmail/Outlook provider、同步状态机、邮件解析和实时策略。它位于 `crates/pebble-mail/src/`，只暴露 crate API，不定义 HTTP 路由。

---

## 目录布局

```text
crates/pebble-mail/src/
├── imap.rs
├── smtp.rs
├── gmail_sync.rs
├── outlook_sync.rs
├── sync.rs
├── parser.rs
└── realtime_policy.rs
```

---

## 模块组织

- `imap.rs`：IMAP 连接、命令、IDLE 和文件夹处理。
- `smtp.rs`：SMTP 发送。
- `gmail_sync.rs`：Gmail History 增量同步。
- `outlook_sync.rs`：Outlook delta 同步。
- `sync.rs`：IMAP 同步状态机。
- `parser.rs`：MIME 解析。
- `realtime_policy.rs`：前后台/失败退避策略。

新增能力优先放入职责最接近的现有模块；只有当现有模块已经承载多个独立状态机或协议边界时才新增文件。

---

## 命名约定

- 文件名使用 snake_case。
- 公共类型名描述领域对象或协议对象，不使用 HTTP/Tauri/RPC 命名。
- 只在 crate 外确实需要时使用 `pub`；crate 内共享优先 `pub(crate)`。

---

## 示例

以当前 `src/lib.rs` 的公共导出和上方模块职责为准；新增模块后同步更新本文件和 `index.md`。
