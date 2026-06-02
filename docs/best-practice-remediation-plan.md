# Pebble 最佳实践问题整改计划（开发执行版）

> 目的：把 Codex 扫描报告与 DeepSeek 扫描报告合并成一份可拆任务、可验收、可追踪的整改计划。本文不是“马上全部重构”的命令，而是“先核验、再止血、再清债”的路线图。

## 0. 执行原则

1. **先验证，再整改**：两份报告都是静态/半静态扫描结果，不能把所有结论当成事实。第 0 阶段必须先复核问题是否真实、是否仍存在、是否有测试保护。
2. **先止血，再翻修**：安全、稳定性、供应链问题优先；模块命名、RPC 透传、大文件拆分等架构债务放在后面。
3. **每个问题必须有验收证据**：不能只说“优化完成”，必须能用测试、命令、文件检查或行为验证证明。
4. **不要为了指标而重构**：例如“纯透传 RPC”不一定都该删除。服务边界有时比少一层函数更重要。
5. **文档和注释使用中文**：遵守 `AGENTS.md`；代码标识符、HTTP 方法、环境变量保留原文。

## 1. 两份报告暴露的问题总览

### 1.1 Codex 报告问题

| ID | 问题 | 影响 | 建议阶段 |
|---|---|---|---|
| C-SEC-01 | 前端依赖存在漏洞：Vitest、Vite、PostCSS、ws | 供应链/开发服务器安全风险 | 第 1 阶段 |
| C-SEC-02 | OAuth 错误页把错误文本直接拼进 HTML | 反射型 XSS 风险 | 第 1 阶段 |
| C-SEC-03 | Docker 基础镜像使用浮动 tag | 构建不可复现，供应链风险 | 第 2 阶段 |
| C-SEC-04 | GitHub Actions action 未 pin 到 SHA | CI 供应链风险 | 第 2 阶段 |
| C-SEC-05 | `.dockerignore` 漏掉 `.agent`、`.claude`、`.codex`、`.antigravitycli` | 本地配置可能进入 Docker build context | 第 1 阶段 |
| C-SEC-06 | nginx `set_real_ip_from 0.0.0.0/0` 信任所有代理头 | 客户端 IP 可被伪造 | 第 1 阶段 |
| C-SEC-07 | CSP 与更新检查/OAuth inline script 冲突 | 生产功能可能被拦截，也可能诱导放宽 CSP | 第 2 阶段 |
| C-META-01 | 版本/仓库元数据漂移：`0.0.10` vs `0.0.4`，`clionertr` vs `QingJ01` | 更新检查、OpenAPI、关于页误导用户 | 第 1 阶段 |
| C-CONTRACT-01 | OpenAPI 与真实路由不同步 | API 集成文档不可信 | 第 2 阶段 |
| C-TOOL-01 | 缺少 ESLint/Prettier | 前端代码风格和 Hooks/a11y 问题不易发现 | 第 2 阶段 |
| C-TOOL-02 | 缺少 Rust 依赖安全/许可证检查 | Rust 漏洞和许可证风险不可见 | 第 2 阶段 |
| C-TOOL-03 | 同时存在 `package-lock.json` 和 `pnpm-lock.yaml` | 依赖锁文件漂移 | 第 2 阶段 |
| C-DOC-01 | Trellis 规范文档大量占位 | AI/新人读到空规范，指导失效 | 第 3 阶段 |
| C-ARCH-01 | 多个巨型文件职责过重 | 后续维护和测试定位困难 | 第 3 阶段 |
| C-ERR-01 | API 内部错误直接暴露给客户端 | 泄露路径、数据库、网络细节 | 第 1/2 阶段 |
| C-HYGIENE-01 | 本地仓库内有 `.env`、`server/data`、`pebble.key` 等运行数据 | 误备份/误打包风险；不是 Git 跟踪问题 | 第 1 阶段 |
| C-ASSET-01 | 根目录 `icon.png` 20MB | 仓库体积膨胀 | 第 3 阶段 |
| C-DOC-02 | README 使用 `curl | bash` 但缺少校验方案 | 安装便利与供应链信任冲突 | 第 2 阶段 |

### 1.2 DeepSeek 报告问题

| ID | 问题 | 影响 | 建议阶段 |
|---|---|---|---|
| D-ARCH-01 | RPC 层有 15 个纯透传函数，另有较多薄 `spawn_blocking` 包装 | 层级可能冗余，维护成本增加 | 第 3 阶段 |
| D-ARCH-02 | API 层有业务逻辑泄漏，尤其 `api/notifications.rs`、`api/auth_api.rs`、`api/attachments.rs`、`api/accounts.rs`、`api/threads.rs` | API handler 变胖，边界不清 | 第 3 阶段，安全相关先处理 |
| D-DEAD-01 | 若干 RPC 函数未被调用或疑似 Tauri 遗留 | 死代码增加理解成本 | 第 2/3 阶段 |
| D-ERR-01 | HTTP 请求可达路径存在 12 处 `.unwrap()` | release profile `panic = "abort"` 时可能导致进程退出 | 第 1 阶段 |
| D-ERR-02 | 关键错误被 `let _ =` 静默吞掉 | 数据脏写、缺少归档文件夹、索引状态不一致难排查 | 第 1 阶段 |
| D-ERR-03 | `imap.rs` 多处 `map_err(|_| ...)` 丢弃原始错误 | 连接/超时排查困难 | 第 2 阶段 |
| D-ERR-04 | 错误类型不一致：部分返回 `String`，`auth_api` 绕过 `ApiError` | 错误模型分裂 | 第 2 阶段 |
| D-DUP-01 | 重复类型：`UserLabel`/`Label`、`StructuredQuery`、搜索查询结构 | 领域模型重复 | 第 3 阶段 |
| D-DUP-02 | 重复查询模式、CSV 解析、Tantivy `SearchHit` 构建 | 小重复累积成维护成本 | 第 3 阶段 |
| D-DUP-03 | `spawn_blocking` 样板重复 16 处 | 错误处理和线程池策略不统一 | 第 3 阶段 |
| D-SEC-01 | 附件上传无请求体大小限制 | 大文件可能造成内存压力/OOM | 第 1 阶段 |
| D-SEC-02 | 附件上传、搜索端点缺少速率/复杂度限制 | CPU/IO 被单用户或恶意请求拖垮 | 第 1/2 阶段 |
| D-SEC-03 | session 仅存内存，过期 session/rate-limit/OAuth state 无定期清理 | 重启登出；长期运行内存增长 | 第 1/2 阶段 |
| D-SEC-04 | `server/.env` 含真实 OAuth 密钥 | 本地安全卫生问题；需确认未入 Git | 第 0/1 阶段 |
| D-SEC-05 | Inbox `limit` 无上限 | 大查询拖垮数据库/内存 | 第 1 阶段 |
| D-STRUCT-01 | request 结构体后缀、模块命名、`pub` 可见性、注释语言等不一致 | 维护体验差 | 第 3 阶段 |
| D-DOC-01 | README、集成指南、SSE/API 文档过时或缺失 | 部署/接入误导 | 第 2 阶段 |
| D-TEST-01 | OAuth、Compose、IMAP、搜索、API 路由、E2E 等覆盖不足 | 回归风险 | 第 2/3 阶段 |

## 2. 需要先校验或不建议照单全收的结论

这些点不是说 DeepSeek/Codex 错了，而是**不适合直接按字面执行**。

| 结论 | 风险 | 处理方式 |
|---|---|---|
| “15 个纯透传 RPC 函数应内联到 API handler” | 可能为了少一层函数破坏 API/RPC/store 三层边界。纯读透传有时是稳定服务边界。 | 第 0 阶段先定义层级职责。只删除确认为遗留/无价值的透传；对仍有边界价值的函数保留但收敛可见性。 |
| “12 个 `.unwrap()` 会让整个服务崩溃” | 在 debug/unwind 下不一定整个进程退出；但当前 release profile 配了 `panic = "abort"`，生产风险成立。 | 按生产配置处理为 P1，全部改成错误传播或静态构建保证。 |
| “OAuth 登录全链路全未测试” | 不完全准确：后端 auth API 有测试，前端 AccountSetup OAuth 有测试；但 provider callback 到账号创建的 E2E 缺口确实存在。 | 计划写成“全链路/E2E 缺失”，不要误写成“完全没有测试”。 |
| “API 路由层只有 1/12 有测试” | 当前 `server/tests/api_test` 覆盖了多个 API 领域，但不是完整路由矩阵。 | 第 0 阶段生成路由-测试覆盖矩阵，用事实替代估算。 |
| “server/.env 含真实 Google OAuth 密钥” | 这不是 Git 跟踪泄漏，但本地仓库放真实密钥仍有误打包/误备份风险。 | 验证 `git ls-files` 无 `.env`；建议迁移到仓库外或开发者私有路径。 |
| “GMAIL_* 缺少 PEBBLE_ 前缀” | 外部集成环境变量有时沿用第三方语义更清晰；改名会影响用户部署。 | 仅在兼容别名方案明确后再改；不能直接破坏现有变量。 |
| “所有英文注释都违规” | `AGENTS.md` 要求生成文档、注释使用中文，但历史英文注释不一定要一次性全改。 | 新增/修改注释使用中文；历史注释在相关文件重构时逐步清理。 |

## 3. 分阶段整改计划与验收标准

### 第 0 阶段：核验报告真伪与建立基线

目标：把“报告说的”变成“仓库证据证明的”。这一阶段不追求修复，追求形成准确清单。

| 整改项 | 覆盖问题 | 执行动作 | 验收标准 |
|---|---|---|---|
| 建立问题核验表 | 全部 | 为本文每个 ID 标记：真实存在、部分存在、已修复、不建议修、需进一步验证 | `docs/best-practice-remediation-plan.md` 或后续 issue 中每个 ID 都有状态；无“未知” |
| 路由-OpenAPI 一致性核验 | C-CONTRACT-01, D-DOC-01 | 写脚本/测试提取 Axum routes 与 `api/docs.rs` paths 做 diff | 测试能列出缺失/多余路径；当前 diff 被记录；后续修复后测试通过 |
| 路由-测试覆盖矩阵 | D-TEST-01 | 列出 `/api/*`、`/events`、`/auth/*`、`/webhook/gmail` 与对应 Rust/前端测试 | 关键 API 至少标出“已有测试/缺测试/不需要测试”的原因 |
| 依赖与安全基线 | C-SEC-01, C-TOOL-02 | 固化 `pnpm audit`、Rust audit/deny 的当前结果 | CI 或本地脚本能重跑；输出归档到任务/issue |
| 本地敏感数据核验 | C-HYGIENE-01, D-SEC-04 | 检查 `.env`、`server/.env`、`data/`、`server/data/` 是否被 Git 跟踪 | `git ls-files` 不包含这些文件；若本地存在，文档提示迁移仓库外 |

### 第 1 阶段：安全与稳定性止血

目标：先处理会导致漏洞、进程退出、资源耗尽、用户误导的高风险问题。

| 整改项 | 覆盖问题 | 执行动作 | 验收标准 |
|---|---|---|---|
| 修复前端依赖漏洞 | C-SEC-01 | 升级 Vite/Vitest/PostCSS/ws 相关依赖；必要时调整测试配置 | `pnpm audit --audit-level moderate` 通过；`pnpm test`、`pnpm run build:frontend` 通过 |
| OAuth HTML 输出转义 | C-SEC-02 | 对 `/auth/login`、`/auth/callback` 中所有用户/外部错误文本做 HTML escape，或改为固定错误页 | 新增测试：`/auth/callback?error=<script>` 响应不包含可执行脚本；`cargo test --workspace --all-targets` 通过 |
| 附件上传大小限制 | D-SEC-01 | 为 `/api/attachments/stage` 加 `DefaultBodyLimit` 或 `RequestBodyLimitLayer`；配置默认值 | 超限上传返回 `413` 或明确错误；正常小文件上传仍通过 |
| 查询 limit 上限 | D-SEC-05 | 对 inbox/thread/search/pending ops 等分页参数设置合理上限 | `limit=100000000` 被 clamp 或返回 400；新增 API 测试覆盖 |
| 搜索/附件基础限流或复杂度限制 | D-SEC-02 | 对高 CPU/IO 端点增加单用户并发上限、查询长度限制、附件数量限制 | 超长搜索词/过多附件返回 400/429；正常搜索不受影响 |
| 消除请求可达 `.unwrap()` | D-ERR-01 | 将 API handler 中 `serde_json::to_value(...).unwrap()` 等改为 `?` / `map_err(ApiError::internal)`；`docs.rs` 使用安全构建 | `git grep` 不再出现报告列出的 12 处 HTTP 可达 unwrap；新增或更新测试验证错误路径 |
| 静默吞错改为可观测 | D-ERR-02 | 对关键 `let _ =` 改成 `if let Err(e)` 并 `warn!`，必要时阻断流程 | 归档文件夹创建失败、账户回滚失败、索引状态写入失败均有日志或返回错误；测试覆盖至少 1 个关键路径 |
| 版本/仓库元数据统一 | C-META-01, D-DOC-01 | 将 UI、OpenAPI、更新检查、CHANGELOG 链接统一到 `package.json/server/Cargo.toml` 与 `clionertr/Pebble` | `git grep '0.0.4\|QingJ01/Pebble'` 仅保留明确“原始上游署名/历史链接”位置；About 页显示 `0.0.10` 或构建注入版本 |
| `.dockerignore` 补齐本地目录 | C-SEC-05 | 增加 `.agent/`、`.claude/`、`.codex/`、`.antigravitycli/`、`.trellis/.backup-*` 等 | `docker build` context 不包含这些目录；可用临时 `tar`/BuildKit 输出或脚本验证 |
| nginx 真实 IP 信任范围收紧 | C-SEC-06 | `set_real_ip_from` 改为具体 Docker 网络/反代 IP，或在示例中说明如何配置 | 默认配置不再信任 `0.0.0.0/0`；README 说明多反代场景配置方式 |
| 本地运行数据迁移建议 | C-HYGIENE-01, D-SEC-04 | 文档说明开发数据应放仓库外，或使用 `.env` 私有路径配置 | README/开发文档含迁移建议；Git 仍不跟踪敏感文件 |

### 第 2 阶段：契约、文档、工具链与供应链

目标：把“能跑”提升为“可持续交付、可集成、可审计”。

| 整改项 | 覆盖问题 | 执行动作 | 验收标准 |
|---|---|---|---|
| OpenAPI 与真实路由同步 | C-CONTRACT-01, D-DOC-01 | 补齐通知、OAuth、snooze/kanban 参数名等；删除不存在路径 | 路由-OpenAPI diff 测试通过；`/api/docs/openapi.json` 包含所有公开入口 |
| 集成指南补齐 API/SSE | D-DOC-01 | 补充推送通知、暂停/收藏/待处理操作、代理、IMAP 测试、翻译流、缺失 SSE 事件 | `docs/integration-guide.md` 的 API/SSE 表覆盖真实公开接口；新增事件有 payload 示例 |
| README 版本和结构修正 | D-DOC-01, C-DOC-02 | 修正 pnpm 版本、项目结构、`site/` 描述、`v0.0.9` 示例；补充 `curl | bash` 校验替代方案 | README 中无过时版本示例；中英文 README 内容一致 |
| CSP 与功能对齐 | C-SEC-07 | 更新检查走后端代理或 CSP 放行 GitHub；OAuth 成功页去 inline script 或加安全 nonce/hash | 生产 CSP 下更新检查和 OAuth 成功跳转可用；无不必要的 `unsafe-inline` 扩散 |
| 前端 lint/format | C-TOOL-01 | 增加 ESLint/Prettier 或等价工具，包含 React Hooks、a11y、no-floating-promises/no-console 约束 | `pnpm lint` 通过；CI 增加 lint；新增代码不产生 lint error |
| Rust 依赖安全/许可证检查 | C-TOOL-02 | 引入 `cargo audit` 或 `cargo deny`，配置忽略策略和许可证策略 | CI 能跑 Rust 依赖检查；已知例外写入配置且有理由 |
| 锁文件统一 | C-TOOL-03 | 移除 `package-lock.json` 或明确 npm 不参与构建；保持 pnpm 单一来源 | CI、Docker、README 均只使用 pnpm；`git ls-files package-lock.json` 为空或有明确保留理由 |
| Docker 镜像 pin | C-SEC-03 | 将基础镜像 pin 到 digest，建立升级流程 | Dockerfile 不再只有浮动 tag；构建仍通过；文档说明如何更新 digest |
| GitHub Actions pin 与产物证明 | C-SEC-04 | 对关键 actions pin SHA；考虑 artifact attestations/SBOM | CI 通过；workflow 权限最小化；发布产物有 checksum，最好有 provenance/SBOM |
| 错误类型统一 | D-ERR-03, D-ERR-04, C-ERR-01 | `rpc/health.rs`、`rpc/diagnostics.rs`、核心 `validate()` 等逐步改为领域错误；API 内部错误改安全文案 | API 对客户端不暴露内部路径/数据库细节；日志保留详细错误；`cargo test` 通过 |
| IMAP 错误保留上下文 | D-ERR-03 | `map_err(|_| ...)` 改为包含原始错误或分类错误 | 日志/错误能区分超时、连接失败、TLS 错误；对应单元测试或集成测试覆盖 |
| 死代码清理 | D-DEAD-01 | 删除 Tauri 遗留或未使用 RPC；保留的函数改成被测试/被调用 | `cargo clippy --workspace --all-targets -- -D warnings` 通过；`git grep` 无明显 Tauri 遗留引用 |
| 关键测试补齐 | D-TEST-01 | 先补 OAuth callback、Compose send、搜索 API、通知 API、OpenAPI diff；再考虑 E2E | 新增测试失败能复现真实风险；全量 Rust/前端测试通过 |

### 第 3 阶段：架构清债与可维护性

目标：在安全和契约稳定后，再处理结构问题。这个阶段不建议“一把梭大重构”，应按模块逐个切。

| 整改项 | 覆盖问题 | 执行动作 | 验收标准 |
|---|---|---|---|
| 明确 API/RPC/store 边界 | D-ARCH-01, D-ARCH-02 | 写入 `.trellis/spec/pebble/backend/directory-structure.md`：API 只做 HTTP 解析，RPC 做业务，store 做持久化 | 新规范存在；后续重构按规范判断，而不是按“函数薄不薄”判断 |
| 处理纯透传 RPC | D-ARCH-01 | 对 15 个函数逐个分类：保留边界、改 `pub(crate)`、合并进更大服务、或删除 | 每个函数有处理结果；不为了减少数量牺牲一致边界；相关 API 测试通过 |
| 拆分胖 API 文件 | D-ARCH-02, D-STRUCT-01 | `api/resources.rs` 拆成 rules/translate/cloud_sync/trusted_senders/templates/diagnostics/proxy；`api/threads.rs` 拆出 search/kanban/snooze | 路由行为不变；OpenAPI diff 测试通过；文件职责更单一 |
| 通知业务下沉 | D-ARCH-02 | 为通知建立 `rpc/notifications.rs` 或 service 层，User-Agent 解析从 handler 移出 | `api/notifications.rs` handler 只做 HTTP 参数提取和响应；通知测试通过 |
| `spawn_blocking` 样板收敛 | D-DUP-03 | 统一使用 `Store::with_read_async/with_write_async` 或新增 helper | 重复 join-error 样板明显减少；错误语义一致 |
| 重复类型与查询模型收敛 | D-DUP-01, D-DUP-02 | 合并 `UserLabel`/`Label`、搜索查询结构、CSV `folderIds` 解析 helper、Tantivy hit builder | 删除未使用类型；新增 helper 有单元测试；调用方行为不变 |
| 巨型文件拆分 | C-ARCH-01, D-STRUCT-01 | 优先拆 `sync.rs`、`provider/gmail.rs`、`provider/outlook.rs`、`sync_cmd.rs`、`AccountsTab.tsx`、`ComposeView.tsx` | 每次拆分保持测试绿；拆分后单文件职责清晰；不引入循环依赖 |
| 可见性收敛 | D-STRUCT-01 | RPC 中仅 API 或同 crate 使用的函数改 `pub(crate)`；真正公开 API 保留 `pub` | `cargo clippy` 和测试通过；无外部 crate 被误断开 |
| 注释语言与规范补齐 | C-DOC-01, D-STRUCT-01 | 填充 Trellis 空规范；新增/修改注释用中文；历史英文注释随重构逐步修 | `.trellis/spec/*` 不再大面积 `To be filled`；关键模块有中文边界说明 |
| 资源文件优化 | C-ASSET-01 | 压缩或替换根目录 20MB `icon.png`，保留必要分辨率 | 图标显示不变；仓库 tracked size 降低；无 UI 回归 |

## 4. 统一质量门

每个阶段完成后至少运行：

```bash
pnpm exec tsc --noEmit
pnpm test
pnpm run build:frontend
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --all-targets
bash -n deploy/install.sh
bash -n deploy/build.sh
```

当第 2 阶段工具链完成后，质量门应扩展为：

```bash
pnpm lint
pnpm audit --audit-level moderate
cargo audit   # 或 cargo deny check
```

## 5. 推荐拆任务顺序

建议按下面顺序拆成独立 PR/任务，避免一次改太多：

1. **依赖漏洞与版本元数据修复**：C-SEC-01、C-META-01。
2. **OAuth XSS + 请求可达 unwrap + 静默吞错**：C-SEC-02、D-ERR-01、D-ERR-02。
3. **资源限制**：附件大小限制、limit 上限、搜索长度/复杂度限制。
4. **Docker/nginx 本地安全卫生**：`.dockerignore`、真实 IP、运行数据文档。
5. **OpenAPI/README/集成指南同步**。
6. **ESLint/Rust audit/锁文件统一/CI 加固**。
7. **API/RPC/store 边界规范写入 Trellis spec**。
8. **拆 `api/resources.rs`、`api/threads.rs`、通知业务下沉**。
9. **重复类型/helper 收敛、纯透传 RPC 分类处理**。
10. **巨型文件逐个拆分与 E2E 补齐**。

## 6. 完成定义

当以下条件都满足，才算“最佳实践整改计划完成”：

- 两份报告中的每个问题 ID 都有状态：已修复、已验证为误报/不建议修、延期且有理由。
- 第 1 阶段安全/稳定性问题全部修复并有测试。
- 第 2 阶段工具链进入 CI，文档/API 契约不再明显漂移。
- 第 3 阶段至少完成高收益结构拆分，并把剩余架构债务记录为明确 backlog。
- 全量质量门通过，工作树干净。

