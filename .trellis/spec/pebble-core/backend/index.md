# Backend Development Guidelines

> pebble-core 的包级后端规范入口。

---

## Overview

`pebble-core` 负责共享领域类型、统一错误、provider trait 和通用 ID/时间工具。实现时先读本目录规范，再结合根项目 `pebble/backend` 的 API/RPC/store 边界。

---

## Guidelines Index

| Guide | Description | Status |
|-------|-------------|--------|
| [Directory Structure](./directory-structure.md) | 模块组织和文件布局 | Done |
| [Quality Guidelines](./quality-guidelines.md) | 质量门、测试和禁用模式 | Done |
| [Error Handling](./error-handling.md) | 错误分类和传播 | Done |
| [Logging Guidelines](./logging-guidelines.md) | 日志级别和敏感信息边界 | Done |
| [Database Guidelines](./database-guidelines.md) | 持久化边界和迁移要求 | Done |

---

## Pre-Development Checklist

- 读 `directory-structure.md`，确认新增代码属于哪个模块。
- 读 `quality-guidelines.md`，确认测试和禁用模式。
- 涉及错误传播时读 `error-handling.md`。
- 涉及持久化、迁移或序列化时读 `database-guidelines.md`。

---

**Language**：文档和注释使用中文；保留代码标识符、HTTP 方法和环境变量原文。
