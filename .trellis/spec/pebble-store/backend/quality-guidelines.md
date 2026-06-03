# 质量指南

> pebble-store 的实现质量约定。

---

## 必守规则

- 所有 SQL 通过 Store 方法封装，使用 with_read/with_write。
- 新增表或字段必须更新 migrations.rs、CURRENT_VERSION 和迁移测试。
- 列表查询必须有明确排序和 limit 上限调用方契约。

---

## 测试要求

- 修改公共 API 或领域行为时，补对应 crate 单元测试。
- 修复 bug 时优先写能复现失败的回归测试。
- 涉及跨层契约时，同步检查 `server` API 测试、OpenAPI 或前端类型。

---

## 禁止模式

- 在库 crate 中直接依赖 `server/src/api`。
- 在请求可达路径新增 `.unwrap()` / `.expect()`，除非测试或不可变常量构造。
- 为通过测试隐藏错误；应保留足够上下文并让调用方决定降级策略。
