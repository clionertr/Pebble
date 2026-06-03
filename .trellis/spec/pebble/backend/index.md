# Backend Development Guidelines

> Best practices for backend development in this project.

---

## Overview

This directory contains guidelines for backend development. Fill in each file with your project's specific conventions.

---

## Guidelines Index

| Guide | Description | Status |
|-------|-------------|--------|
| [Configuration](./configuration.md) | Environment variables and binding host | Done |
| [Directory Structure](./directory-structure.md) | Module organization and file layout | Done |
| [Webmail API Contracts](./webmail-api-contracts.md) | REST/SSE/auth/deploy cross-layer contracts | Done |
| [Database Guidelines](./database-guidelines.md) | ORM patterns, queries, migrations | Done (Phase 3) |
| [Error Handling](./error-handling.md) | Error types, handling strategies | Done (Phase 1) |
| [Quality Guidelines](./quality-guidelines.md) | Code standards, forbidden patterns, security patterns | Done (Phase 1) |
| [Logging Guidelines](./logging-guidelines.md) | Structured logging, log levels | Done (Phase 3) |

---

## How to Fill These Guidelines

For each guideline file:

1. Document your project's **actual conventions** (not ideals)
2. Include **code examples** from your codebase
3. List **forbidden patterns** and why
4. Add **common mistakes** your team has made

The goal is to help AI assistants and new team members understand how YOUR project works.

---

**Language**: 文档和注释使用中文；保留代码标识符、HTTP 方法和环境变量原文。
