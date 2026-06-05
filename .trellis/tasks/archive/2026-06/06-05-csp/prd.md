# 修复生产 CSP 下邮件内联样式拦截

## Goal

让 Docker/nginx 生产环境在保持 CSP 收敛的前提下，允许经过 sanitizer 白名单过滤的邮件 `style=""` 属性生效，修复邮件内容和 blocked image placeholder 样式被浏览器拦截的问题。

## What I already know

- `deploy/nginx.conf` 当前使用 `style-src 'self'`，生产环境会阻止元素上的 `style=""`。
- `pnpm run dev:frontend` 没有生产 CSP，因此邮件 CSS 正常。
- `crates/pebble-privacy/src/sanitizer.rs` 当前允许 `style` 通用属性，并通过 CSS 属性和值白名单过滤危险内容。
- 最新提交 `504ff87` 为 blocked image placeholder 新增了 `style="width:...;height:..."`，使生产 CSP 冲突更明显。

## Requirements

- 将生产 nginx CSP 从粗粒度 `style-src 'self'` 调整为：
  - `style-src 'self'`
  - `style-src-elem 'self'`
  - `style-src-attr 'unsafe-inline'`
- 不使用全局 `style-src 'self' 'unsafe-inline'`。
- 为 sanitizer 增加回归测试，覆盖危险内联样式不会被保留：
  - `url()`
  - `data:`
  - `javascript:`
  - `@import`
  - 反斜杠转义
  - 不允许的布局属性如 `position` / `z-index`

## Acceptance Criteria

- [x] `deploy/nginx.conf` 使用 `style-src-attr 'unsafe-inline'`，同时 `style-src-elem` 保持 `self`。
- [x] sanitizer 回归测试证明危险 CSS 值和不在白名单内的属性被过滤。
- [x] 相关测试通过。
- [x] 提交一个修复 commit。

## Out of Scope

- 不重构邮件 Shadow DOM 渲染。
- 不移除 sanitizer 对安全邮件内联样式的支持。
- 不修改 README 示例配置，除非实现验证发现必须同步。

## Technical Notes

- 关键文件：
  - `deploy/nginx.conf`
  - `crates/pebble-privacy/src/sanitizer.rs`
  - `src/components/ShadowDomEmail.tsx`
