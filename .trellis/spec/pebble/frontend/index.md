# Frontend Development Guidelines

> Pebble 前端启动、应用壳和浏览器边界的实现约定。

---

## Overview

Pebble 前端是 React SPA，源码位于 `src/`，HTML 启动壳位于 `index.html`。
涉及浏览器先于 React 执行的逻辑时，以本目录的具体规范为准。

---

## Guidelines Index

| Guide | Description | Status |
|-------|-------------|--------|
| [Startup UI](./startup-ui.md) | 启动 splash 与 React 挂载边界 | Done |

---

## Pre-Development Checklist

- 修改 `index.html`、`src/main.tsx`、`src/App.tsx` 或启动打点时，先读 [Startup UI](./startup-ui.md)。
- 修改全局 `window` API 时，同步更新 `src/vite-env.d.ts`。

---

## Quality Check

- 跑 `pnpm exec tsc --noEmit`。
- 跑 `pnpm lint`。
- 触及 `index.html` 时跑 `pnpm build:frontend`，确认 Vite 能处理生产构建。

---

**Language**: 文档和注释使用中文；保留代码标识符、事件名和全局 API 原文。
