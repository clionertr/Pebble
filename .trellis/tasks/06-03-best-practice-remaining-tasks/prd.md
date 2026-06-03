# 最佳实践剩余任务

## 目标

完成用户要求的“最佳实践剩余任务表”中的任务。该目标范围较大，需要分批推进，但最终状态应使表中任务全部完成或有明确、可验收的当前状态。

## 本轮优先级

优先做能提高后续验收确定性的 P1/P2 基础项：

1. 更新 `docs/best-practice-remediation-plan.md` 的当前状态表，避免已修复项仍显示为真实存在。
2. 增加 OpenAPI 路由 diff 自动测试，防止真实路由和 `server/src/api/docs.rs` 再次漂移。
3. 增加 Rust 依赖安全/许可证检查入口，优先选 `cargo audit` 或 `cargo deny` 中适合当前仓库的方案。
4. 推进错误类型统一和 IMAP 错误上下文保留的第一批高价值改造。
5. 继续收敛 `spawn_blocking` 样板和纯透传 RPC 可见性。

## 剩余任务表

| 优先级 | 任务 | 验收标准 |
|---|---|---|
| P1 | 更新问题状态总表 | 每个 ID 标记当前状态；已修复项不再误显示为待修 |
| P1 | OpenAPI 路由 diff 自动测试 | 测试提取 Axum routes 与 OpenAPI paths 对比；CI/本地测试通过 |
| P1 | Rust 依赖安全/许可证检查进入 CI | CI 或脚本能跑 `cargo audit` / `cargo deny check`；例外有说明 |
| P1 | 错误类型统一 | 内部错误不暴露给客户端；`rpc/health.rs`、`rpc/diagnostics.rs` 等不再散落 `String` 错误 |
| P1 | IMAP 错误保留上下文 | 连接、超时、TLS、认证等错误保留原始上下文或分类 |
| P1 | 关键 API 测试补齐 | OAuth callback、Compose send、搜索 API、通知 API、OpenAPI diff 有回归测试 |
| P2 | 拆 `api/resources.rs` | 按资源域拆分，路由行为不变 |
| P2 | 拆 `api/threads.rs` | search/kanban/snooze 拆分，先抽共享查询 helper |
| P2 | 继续收敛 `spawn_blocking` 样板 | 旧 join-error 转换明显减少，测试通过 |
| P2 | 纯透传 RPC 分类和可见性收敛 | 每个薄 RPC 有结论：保留、`pub(crate)`、合并或删除 |
| P2 | GitHub Actions 产物证明/SBOM | release 产物有 checksum，最好有 provenance/SBOM |
| P3 | 巨型同步/Provider 文件拆分 | 先补测试，再按状态机、协议请求、转换、错误分类拆 |
| P3 | 前端巨型组件拆分 | `AccountsTab.tsx`、`ComposeView.tsx` 按职责拆分，前端测试通过 |
| P3 | Trellis 包级占位规范清理 | `.trellis/spec/*` 不再大面积 `To be filled by the team` |
| P3 | E2E 覆盖 | 核心用户流 E2E 覆盖真实风险 |

## 暂不降级

用户要求完成整张表，不把“完成第一批”当作最终完成。每轮结束时需要记录哪些任务已完成、哪些仍剩余。

## 质量门

- `cargo fmt --all -- --check`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace --all-targets`
- `pnpm exec tsc --noEmit`
- `pnpm test`
- `pnpm run build:frontend`
- `pnpm audit --audit-level moderate`
- `bash -n deploy/install.sh && bash -n deploy/build.sh`

