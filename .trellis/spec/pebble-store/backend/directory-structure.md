# 目录结构

> pebble-store 后端代码组织约定。

---

## 总览

`pebble-store` 负责SQLite 持久化、迁移、查询和事务边界。它位于 `crates/pebble-store/src/`，只暴露 crate API，不定义 HTTP 路由。

---

## 目录布局

```text
crates/pebble-store/src/
├── lib.rs
├── migrations.rs
├── messages.rs
├── accounts.rs
├── pending_ops.rs
├── notification_devices.rs
└── secure_user_data.rs
```

---

## 模块组织

- `lib.rs`：Store、连接池/读写封装。
- `migrations.rs`：schema 版本迁移。
- `messages.rs`：消息/线程/文件夹关联。
- `accounts.rs`：账号和同步游标。
- `pending_ops.rs`：离线远端操作队列。
- `notification_devices.rs`：Web Push 设备。
- `secure_user_data.rs`：加密用户配置 blob。

新增能力优先放入职责最接近的现有模块；只有当现有模块已经承载多个独立状态机或协议边界时才新增文件。

---

## 命名约定

- 文件名使用 snake_case。
- 公共类型名描述领域对象或协议对象，不使用 HTTP/Tauri/RPC 命名。
- 只在 crate 外确实需要时使用 `pub`；crate 内共享优先 `pub(crate)`。

---

## 示例

以当前 `src/lib.rs` 的公共导出和上方模块职责为准；新增模块后同步更新本文件和 `index.md`。
