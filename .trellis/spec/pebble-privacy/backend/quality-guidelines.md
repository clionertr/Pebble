# 质量指南

> pebble-privacy 的实现质量约定。

---

## 必守规则

- 默认策略必须偏保守：未知 URL、script、iframe、事件属性和危险 CSS 一律阻止。
- 隐私模式变更必须覆盖 Strict、LoadOnce、TrustSender、Off 的差异测试。
- 不要引入需要执行 HTML/JS 的解析逻辑。

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
