# Database Guidelines

> Database patterns and conventions for this project.

---

## Overview

Pebble 使用 `rusqlite` 直接访问 SQLite，不使用 ORM。数据库访问集中在
`crates/pebble-store/src/`，`server/src/api/` 不直接写 SQL。

`Store` 内部维护读连接池和写连接池：

- 同步 store 方法通过 `with_read` / `with_write` 获取连接。
- async service 或后台任务调用 store 时，使用 `with_read_async`、`with_write_async` 或 `with_blocking_async`，避免阻塞 Tokio runtime。
- 复杂业务先放在 service 层编排，SQL 查询本身放在 `pebble-store`。

---

## Query Patterns

- 查询函数返回领域类型或明确的投影类型，不在 API handler 中拼 SQL。
- 批量读取优先提供专门方法，例如 `get_messages_batch`、`get_message_labels_batch`，避免 API 层循环逐条查库。
- 列表查询必须有分页上限；API 层负责 clamp 外部 `limit`，store 层仍按传入值执行。
- 写入使用 `INSERT OR IGNORE`、事务或显式回滚时，错误不得静默吞掉；确实可忽略的清理错误要写注释说明。

---

## Migrations

迁移位于 `crates/pebble-store/src/migrations.rs`，由 `Store::open` 初始化时执行。

- 新增表、列、索引时更新迁移序列，并补充至少一个 store 层测试或 API 行为测试。
- 迁移必须幂等：重复打开已有数据库不应失败。
- 不在运行时 handler 中临时建表或补 schema。

---

## Naming Conventions

- 表名使用小写 snake_case 复数或领域集合名，例如 `messages`、`notification_devices`。
- 字段名使用 snake_case；跨 HTTP 边界时由 serde 映射成 camelCase。
- Rust 领域类型使用清晰名词；同字段模型只保留一个权威类型，避免 `UserLabel` / `Label` 这类重复定义再次漂移。
- store 方法名描述业务动作，例如 `pause_expired_notification_devices`，不要暴露 SQL 细节。

---

## Common Mistakes

- 在 async handler 中直接调用同步 SQLite 方法，会阻塞 runtime；应通过 service 层使用异步 blocking helper。
- 为每个调用方复制一份查询结构，后续字段新增时容易漏改；优先复用 `pebble-core` 类型或 store 投影类型。
- 把数据库错误原文返回客户端会泄露路径、SQL 或内部状态；API 层只返回安全文案，日志保留细节。
