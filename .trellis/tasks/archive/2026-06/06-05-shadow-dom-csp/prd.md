# 修复 Shadow DOM 邮件样式 CSP 拦截

## Goal

修复生产环境 `style-src-elem 'self'` 下，`ShadowDomEmail` 运行时创建内联 `<style>` 元素被浏览器 CSP 拦截的问题。

## What I already know

- 线上 `https://pebble.ailolis.net/` 返回 CSP：`style-src 'self'; style-src-elem 'self'; style-src-attr 'unsafe-inline'`。
- 浏览器报错是 `style-src-elem 'self'`，不是 `style-src-attr`。
- `src/components/ShadowDomEmail.tsx` 当前通过 `document.createElement("style")` + `textContent` 创建 Shadow DOM 基础样式。
- 生产 CSP 禁止内联 `<style>`，即使它由 JS 创建也会被拦截。
- 同源外部 CSS 文件符合 `style-src-elem 'self'`。

## Requirements

- 移除 `ShadowDomEmail` 中运行时创建内联 `<style>` 的实现。
- 将 Shadow DOM 邮件基础样式放到同源静态 CSS 文件中。
- 在 Shadow DOM 内用 `<link rel="stylesheet" href="...">` 加载该 CSS。
- 保持邮件正文 `style=""` 属性继续由 `style-src-attr 'unsafe-inline'` + sanitizer 白名单保护。

## Acceptance Criteria

- [x] `ShadowDomEmail` 不再创建内联 `<style>` 元素。
- [x] 邮件 Shadow DOM 样式从同源 CSS 文件加载。
- [x] `pnpm exec tsc --noEmit` 通过。
- [x] `pnpm build:frontend` 通过。
- [x] 提交修复 commit。

## Out of Scope

- 不调整 CSP 策略。
- 不重构邮件 sanitizer。
- 不修改 1Panel/OpenResty 反向代理配置。

## Technical Notes

- 关键文件：
  - `src/components/ShadowDomEmail.tsx`
  - `public/`
  - `deploy/nginx.conf`
