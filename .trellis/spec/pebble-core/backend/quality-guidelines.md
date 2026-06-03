# 质量指南

> pebble-core 的实现质量约定。

---

## 必守规则

- 保持无外部运行时依赖，类型必须可序列化/反序列化时显式派生 serde。
- 新增跨层 DTO 时同步检查前端 API 类型、OpenAPI 和 store 映射。
- 不要在 core 中引入 server、store、mail 等反向依赖。

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
