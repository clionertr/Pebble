# 数据库指南

> pebble-core 的持久化边界。

---

## 边界

本包不访问数据库；只定义被 pebble-store 持久化的结构。字段调整必须同步迁移、store 读写和 API/前端契约。

---

## 契约

- 直接 SQL 只允许在 `pebble-store` 中出现。
- 跨 crate 数据结构调整必须同步 store 映射、迁移或调用方序列化。
- 加密配置和用户数据以密文 blob 存储，明文只在调用边界短暂存在。

---

## 测试

- 持久化结构变更要有 round-trip 测试。
- 迁移变更要验证旧版本数据可升级到 `CURRENT_VERSION`。
- 无数据库依赖的 crate 要通过单元测试证明行为，不引入 SQLite fixture。
