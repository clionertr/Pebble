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

---

## 附录 B：第 3 阶段落地记录

> 落地日期：2026-06-03
> 执行原则：先做低风险、高收益、可回滚的小切口；大型模块拆分记录为后续 backlog，不在一次任务里“一把梭”。

### B.1 已完成项

| 整改项 | 覆盖问题 | 处理结果 | 验收证据 |
|---|---|---|---|
| 明确 API/RPC/store 边界 | D-ARCH-01, D-ARCH-02 | 已写入 `.trellis/spec/pebble/backend/directory-structure.md`，明确 API 是 HTTP 适配层，`server/src/rpc/` 是内部 service 层，store/crates 负责持久化和领域能力 | 规范包含“API/RPC/store 边界”“允许保留的薄服务函数”“阻塞 I/O 约定” |
| 通知业务下沉 | D-ARCH-02 | `api/notifications.rs` 不再承担设备名推断、订阅注册业务和首次未读摘要发送；相关逻辑迁入 `rpc/notifications.rs` | handler 只提取 Cookie/Header/Json 并调用 service；通知 API 路由不变 |
| 重复查询模型收敛 | D-DUP-01 | 高级搜索复用 `pebble_core::traits::StructuredQuery`，`server/src/rpc/advanced_search.rs` 不再维护同字段结构 | `AdvancedSearchQuery` 变为核心类型别名，serde camelCase 契约保留 |
| 标签重复类型收敛 | D-DUP-01 | `pebble-store::labels::Label` 改为 `pebble_core::UserLabel` 的类型别名，避免 store/API 与 core 同字段模型漂移 | 调用方类型名保持 `Label`，序列化结构不变 |
| Tantivy hit builder 收敛 | D-DUP-02 | `pebble-search` 新增 `search_hit_from_doc()`，普通搜索和高级搜索共用命中构建逻辑 | 删除两处重复的 `SearchHit` 字段拼装代码 |
| `spawn_blocking` 样板收敛 | D-DUP-03 | `Store` 新增 `with_blocking_async()`，线程、消息、文件夹、未读计数等纯读 RPC 改用统一 helper | join-error 转换集中到 store helper；后续旧服务函数按同模式逐步迁移 |
| 注释语言与规范补齐 | C-DOC-01, D-STRUCT-01 | 填充后端 database/logging 规范，占位状态更新为 Done；新增注释使用中文 | `.trellis/spec/pebble/backend/database-guidelines.md`、`logging-guidelines.md` 不再包含占位正文 |
| 资源文件优化 | C-ASSET-01 | 根目录 `icon.png` 从 5040x5036、20MB 压缩/重采样为 1024x1023、约 703KB | `file icon.png` 显示 1024x1023；`ls -lh icon.png` 显示约 703KB |

### B.2 已评估但延期的架构债务

| Backlog | 延期原因 | 建议后续拆法 |
|---|---|---|
| 拆 `api/resources.rs` | 该文件挂载 rules/translate/cloud-sync/trusted-senders/templates/diagnostics/proxy 多个路由，机械拆分容易造成 OpenAPI、前端客户端和测试同时漂移 | 单独开任务，先建立 route snapshot/OpenAPI diff，再按资源域拆模块并保持 `resource_routes()` 聚合入口 |
| 拆 `api/threads.rs` | threads/search/kanban/snooze 共用分页和搜索契约，拆分时容易影响前端 Inbox/Search/Kanban 多处调用 | 先抽 `parse_folder_ids` 和搜索请求类型，再按 search/kanban/snooze 分文件 |
| 巨型同步/Provider 文件拆分 | `sync.rs`、`provider/gmail.rs`、`provider/outlook.rs` 涉及协议状态机、重试、增量同步和错误恢复，缺少足够 E2E 保护时不适合一次性重排 | 先补同步集成测试和 provider fake，再按“状态机/协议请求/消息转换/错误分类”拆 |
| 纯透传 RPC 全量改可见性 | 当前 `server/src/rpc/` 作为内部 service 层，薄函数仍有边界价值；盲目删除会让 API handler 直接依赖 store 细节 | 逐模块评估：有编排潜力保留，单调用方且无边界价值改 `pub(crate)` 或合并 |

### B.3 第 3 阶段当前状态

第 3 阶段已完成一批高收益清债，并把大规模重构收敛为后续 backlog。后续若继续推进，推荐顺序是：

1. `api/resources.rs` 拆分，配套 OpenAPI diff 和 API 测试。
2. `api/threads.rs` 拆分，先抽共享查询解析 helper。
3. `sync.rs` / provider 文件拆分，先补协议假实现和同步回归测试。
4. 继续把旧 `spawn_blocking` 样板迁移到统一 helper。

---

## 附录 C：当前剩余任务状态表

> 更新日期：2026-06-03
> 说明：附录 A 是第 0 阶段基线快照，保留历史证据；本表反映当前执行状态，后续任务以本表为准。

| 优先级 | 任务 | 当前状态 | 下一步验收标准 |
|---|---|---|---|
| P1 | 更新问题状态总表 | **已完成当前映射**：附录 C.1 已按 36 个问题 ID 逐项映射到“已完成 / 部分完成 / 剩余”等当前状态，避免直接把附录 A 基线当作最新事实 | 后续执行每完成一项即更新 C.1；已修复项不再误导后续执行 |
| P1 | OpenAPI 路由 diff 自动测试 | **已完成**：已新增 `api::docs::tests::openapi_paths_match_public_routes`，用于扫描真实路由并比较 OpenAPI paths | `cargo test -p pebble openapi_paths_match_public_routes -- --nocapture` 通过；后续新增公开路由缺文档会失败 |
| P1 | Rust 依赖安全/许可证检查进入 CI | **已完成 CI 接入**：`deny.toml` 存在，CI 已加入 SHA pin 的 `EmbarkStudios/cargo-deny-action`；本机当前未安装 `cargo-deny`，无法直接跑本地复核 | CI cargo-deny job 通过；已知例外写入 `deny.toml` |
| P1 | 错误类型统一 | **已完成当前 API 边界**：`rpc/health.rs`、`rpc/diagnostics.rs` 已迁到 `PebbleError`，`auth_api` 已改走 `ApiError`，`record_timing` 不再向客户端拼接内部错误；新增静态测试防止 `/api` handler 回退到 `Result<..., String>` 或裸 `StatusCode + Json` | `cargo test -p pebble --test api_baseline api_handlers_do_not_bypass_api_error_boundary -- --nocapture` 通过；客户端只见安全文案，日志保留内部细节 |
| P1 | IMAP 错误保留上下文 | **已完成当前范围**：IMAP 测试连接的 TCP/SOCKS5/TLS/greeting 超时错误带目标地址；深层 SELECT/FETCH/STORE/MOVE/IDLE 等命令统一经 `with_imap_timeout()` 保留操作名和原始错误；IMAP UID 解析错误也保留 `ParseIntError` 上下文 | `rg 'map_err\(\|_\|' crates/pebble-mail/src/imap.rs server/src/rpc/messages/provider_dispatch.rs` 无输出；相关单测通过 |
| P1 | 关键 API 测试补齐 | **已完成 API 范围**：已有 API baseline、auth、OAuth callback、Compose send、messages、shell、snooze、trusted_senders、搜索、通知、草稿、标签、偏好设置、诊断、日志、代理测试；OpenAPI diff 测试已补入 | `cargo test -p pebble --test api_test -- --nocapture` 41 个测试通过；端到端覆盖继续作为 P3 E2E 项推进 |
| P2 | 拆 `api/resources.rs` | **已完成**：`resources.rs` 已改为聚合路由，rules/translate/contacts/cloud_sync/trusted_senders/templates/preferences/diagnostics/proxy 已拆到 `server/src/api/resources/` 子模块 | 递归 OpenAPI route diff、API 测试、全量 Rust 测试通过；路由行为不变 |
| P2 | 拆 `api/threads.rs` | **已完成**：`threads.rs` 已改为聚合路由，thread_reads/search/kanban/snooze 已拆到 `server/src/api/threads/` 子模块，共享分页上限保留在聚合模块 | 递归 OpenAPI route diff、API 测试、全量 Rust 测试通过；路由行为不变 |
| P2 | 继续收敛 `spawn_blocking` 样板 | **已完成 RPC 层收敛**：已新增 `Store::with_blocking_async()` 和 `rpc::blocking::run_blocking()`；search/advanced_search、messages flags/rendering、batch、attachments、accounts cleanup、reindex 等旧 join-error 样板已迁移 | `rg 'tokio::task::spawn_blocking|Task join error' server/src/rpc` 仅剩统一 helper；全量 Rust 质量门通过 |
| P2 | 纯透传 RPC 分类和可见性收敛 | **已完成当前清单**：薄 RPC 已按“保留边界 / 收窄可见性 / 删除遗留透传”分类；labels/rules/kanban/snooze/threads/messages query 等仅 crate 内调用的函数已改 `pub(crate)`，未使用的 `get_global_proxy()` 已删除 | `cargo clippy --workspace --all-targets -- -D warnings` 与 API 测试通过；分类结果见 C.2 |
| P2 | GitHub Actions 产物证明/SBOM | **已完成 checksum 基线**：Actions 已 SHA pin，Docker digest pin 已完成；release 二进制逐平台生成 `.sha256`，发布前统一校验并上传 `checksums.txt` | 后续可继续评估 artifact attestations/SBOM；当前 release 产物完整性校验已有可验收基线 |
| P3 | 巨型同步/Provider 文件拆分 | **已完成当前范围**：Gmail provider 已抽出 MIME/地址/附件编码 helper 到 `gmail_mime.rs`；Outlook provider 已抽出 base64/MIME helper 到 `outlook_codec.rs`；`sync_cmd.rs` 已抽出实时状态、wake 统计、触发决策到 `sync_cmd_support.rs`，主文件由 1196 行降到 764 行；`sync.rs` 已抽出附件落盘到 `sync_attachments.rs`、IMAP cursor/轮询/错误分类到 `sync_imap_state.rs`，主文件由 2260 行降到 1603 行 | `cargo test -p pebble rpc::sync_cmd -- --nocapture` 15 个测试通过；`cargo test -p pebble-mail sync -- --nocapture` 52 个测试通过；`cargo clippy -p pebble --all-targets -- -D warnings` 与 `cargo clippy -p pebble-mail --all-targets -- -D warnings` 通过 |
| P3 | 前端巨型组件拆分 | **已完成当前范围**：`ComposeView.tsx` 已抽出附件列表、模板菜单、保存模板面板、离开确认弹窗到 `ComposePanels.tsx`，主文件由 1118 行降到 794 行；`AccountsTab.tsx` 已抽出账号列表到 `AccountsList.tsx`、编辑弹窗到 `EditAccountModal.tsx`，主文件由 1237 行降到 280 行 | `pnpm lint`、Accounts 相关测试、`pnpm build:frontend` 通过；后续新增功能继续按组件边界拆 |
| P3 | Trellis 包级占位规范清理 | **已完成**：`pebble-core`、`pebble-crypto`、`pebble-mail`、`pebble-oauth`、`pebble-privacy`、`pebble-rules`、`pebble-search`、`pebble-store`、`pebble-translate` 的包级 backend spec 已替换为真实目录/质量/错误/日志/数据库边界规范 | `rg '(To be filled by the team)' .trellis/spec -g '*.md'` 无输出；每个包至少有真实目录/质量/错误规范入口 |
| P3 | E2E 覆盖 | **已完成当前 Vitest 范围**：新增 `tests/e2e/` 核心流测试，覆盖 OAuth 账号入口、Compose typed recipient 到发送 mutation、搜索提交到结果详情打开 | `pnpm test` 79 文件 / 274 测试通过；后续如引入 Playwright 再补真实浏览器后端联调 |

### C.1 问题 ID 当前状态

| ID | 当前状态 | 当前证据 / 下一步 |
|---|---|---|
| C-SEC-01 | **已完成** | `pnpm audit --audit-level moderate` 已为零漏洞；依赖升级已固化在第 1/2 阶段。 |
| C-SEC-02 | **已完成** | `server/src/auth.rs` 使用 `escape_html()`，OAuth 错误页测试覆盖脚本转义。 |
| C-SEC-03 | **已完成** | `deploy/backend.Dockerfile`、`deploy/frontend.Dockerfile` 的 `FROM` 均使用 `@sha256` digest pin。 |
| C-SEC-04 | **已完成** | `.github/workflows/*.yml` action 均用 SHA pin，并保留 tag 注释。 |
| C-SEC-05 | **已完成** | `.dockerignore` 已包含 `.agent`、`.claude`、`.codex`、`.antigravitycli`、`server/data` 等本地目录。 |
| C-SEC-06 | **已完成** | `deploy/nginx.conf` 不再信任 `0.0.0.0/0`，改为 Docker 私网 CIDR 和 loopback。 |
| C-SEC-07 | **已完成当前范围** | OAuth 成功页和 API docs 已去除 inline style；nginx CSP 已收紧为 `style-src 'self'`，不再允许 `'unsafe-inline'`。 |
| C-META-01 | **已完成** | OpenAPI、更新检查、About 页、站点链接、CHANGELOG compare 链接已对齐 `0.0.10` 与 `clionertr/Pebble`；README 仅保留明确的原始上游说明和署名。 |
| C-CONTRACT-01 | **已完成** | OpenAPI 已补通知路由，新增 `openapi_paths_match_public_routes` 自动 diff 测试。 |
| C-TOOL-01 | **已完成** | ESLint/Prettier 严格门禁已可通过；CI 前端 lint/format 不再 `continue-on-error`；`pnpm lint`、`pnpm format:check`、`pnpm test`、`pnpm build:frontend` 均通过。 |
| C-TOOL-02 | **已完成 CI 接入** | `deny.toml` 与 SHA pin 的 `cargo-deny-action` 已存在；本机未安装 `cargo-deny`，本地复核依赖开发环境。 |
| C-TOOL-03 | **已完成** | `git ls-files package-lock.json` 为空，仓库只保留 `pnpm-lock.yaml`。 |
| C-DOC-01 | **已完成当前范围** | 主 `pebble/backend` 规范已补；包级 backend spec 占位正文已清理，目录/质量/错误/日志/数据库边界均有真实内容。 |
| C-ARCH-01 | **已完成当前范围** | `api/resources.rs`、`api/threads.rs` 已拆；`ComposeView.tsx` 已完成面板拆分；`AccountsTab.tsx` 已抽出账号列表和编辑弹窗；Gmail/Outlook provider 已抽出 MIME/编码 helper；`sync_cmd.rs` 已抽出 `sync_cmd_support.rs`；`sync.rs` 已抽出 `sync_attachments.rs` 与 `sync_imap_state.rs`。剩余 `oauth.rs`、`accounts.rs`、`batch.rs` 属中等偏大，未在本轮表格范围内继续拆。 |
| C-ERR-01 | **已完成当前 API 边界** | `ApiError` 默认内部错误已脱敏，`record_timing` 已改安全返回；`api_handlers_do_not_bypass_api_error_boundary` 防止新增 `/api` handler 绕过 `ApiError`。 |
| C-HYGIENE-01 | **已完成** | Git 不跟踪 `.env`、`data/`、`server/data/`、`pebble.key`，`.dockerignore` 已排除本地运行数据。 |
| C-ASSET-01 | **已完成** | 根目录 `icon.png` 已从 20MB 压缩到约 703KB。 |
| C-DOC-02 | **已完成** | README 中 `curl | bash` 已补校验替代方案，中英文 README 已同步。 |
| D-ARCH-01 | **已完成当前清单** | 已写入 API/RPC/store 边界规范；纯透传 RPC 已完成分类和可见性收敛，保留的薄函数作为 API → service 边界存在。 |
| D-ARCH-02 | **已完成当前 API 拆分范围** | 通知业务已下沉到 service；`api/resources.rs` 与 `api/threads.rs` 已按资源域拆分，路由行为由递归 OpenAPI diff 和 API 测试保护。 |
| D-DEAD-01 | **已完成** | Tauri 遗留引用已清理，API/RPC inventory 测试覆盖无 `/rpc` 暴露。 |
| D-ERR-01 | **已完成** | 请求可达 `.unwrap()` 已清理到安全位置或测试代码；质量门包含 clippy 和全量测试。 |
| D-ERR-02 | **已完成服务端审视** | 关键搜索 pending、账号回滚、归档文件夹种子、IMAP 断开、Gmail realtime 错误记录等业务路径已有日志或显式错误传播；服务端剩余 `let _ =` 仅为事件广播、临时文件/测试目录清理。 |
| D-ERR-03 | **已完成当前范围** | IMAP 测试连接超时已带目标地址；深层 IMAP 命令经 `with_imap_timeout()` 保留操作名和原始错误；`parse_imap_uid()` 已保留解析错误上下文。 |
| D-ERR-04 | **已完成** | `rpc/health.rs`、`rpc/diagnostics.rs` 已迁到 `PebbleError`；`api/auth_api.rs` 登录错误已统一返回 `ApiError`，响应 JSON shape 保持 `{ "error": ... }`。 |
| D-DUP-01 | **已完成当前范围** | `pebble-store::labels::Label` 已是 `pebble_core::UserLabel` 类型别名；`AdvancedSearchQuery` 已是 `StructuredQuery` 类型别名。`rg 'struct (UserLabel\|Label\|StructuredQuery\|AdvancedSearchQuery)' crates server/src` 仅保留核心定义和类型别名。 |
| D-DUP-02 | **已完成当前范围** | Tantivy `SearchHit` 构建已收敛到 `search_hit_from_doc()`；`folderIds` CSV 解析已抽到 `api::query::parse_csv_query_ids()` 并由 messages/threads 共用；`cargo test -p pebble api::query -- --nocapture` 通过。 |
| D-DUP-03 | **已完成 RPC 层收敛** | 已新增 `Store::with_blocking_async()` 与 `rpc::blocking::run_blocking()`；`server/src/rpc` 旧 `Task join error` 样板已清零，直接 `spawn_blocking` 仅剩统一 helper。非 RPC 后台 worker 中的阻塞任务按运行时职责保留。 |
| D-SEC-01 | **已完成** | `/api/attachments/stage` 已使用 `DefaultBodyLimit::max(MAX_ATTACHMENT_SIZE)` 并做 handler 内二次检查。 |
| D-SEC-02 | **已完成基础限制** | 搜索查询长度、分页 limit 已限制；搜索与附件上传会抢占全局并发 permit，过载时返回 429。更细粒度的单用户配额可作为后续增强，不再阻塞本整改项。 |
| D-SEC-03 | **已完成内存清理** | `SessionStore` 已提供 session/rate-limit 过期清理并在启动时挂后台任务；OAuth state 已记录创建时间并按 TTL 定期清理。当前单用户 session 仍为内存态，重启登出作为已知部署取舍保留。 |
| D-SEC-04 | **已完成** | `server/.env` 未被 Git 跟踪，`.dockerignore` 已排除本地敏感数据。 |
| D-SEC-05 | **已完成** | inbox/thread/search/pending ops 等 limit 已 clamp 到上限。 |
| D-STRUCT-01 | **已完成当前范围** | 新增规范和部分可见性/注释改造已完成；`ComposeView.tsx` 已抽出面板组件；`AccountsTab.tsx` 已抽出账号列表和编辑弹窗；同步 RPC/IMAP 附件/IMAP 状态逻辑已拆为职责更清晰的内部模块。历史英文注释随后续功能重构逐步修，不再阻塞本表格任务。 |
| D-DOC-01 | **已完成基础同步** | README、集成指南、OpenAPI 已大幅补齐；后续随新增 API/SSE 继续维护。 |
| D-TEST-01 | **已完成当前范围** | API 层已有 baseline/auth/OAuth callback/Compose send/messages/shell/snooze/trusted_senders/search/notifications/drafts/labels/preferences/diagnostics/logs/proxy/OpenAPI diff 测试；前端新增 Vitest 核心流覆盖 OAuth 账号入口、Compose 发送、搜索结果详情。 |

### C.2 纯透传 RPC 分类结果

| 模块 / 函数组 | 处理结论 | 证据 |
|---|---|---|
| `rpc/labels.rs` | 保留 service 边界，全部改 `pub(crate)` | 仅 `api/labels.rs` 调用；handler 不直接依赖 store 标签细节。 |
| `rpc/rules.rs` | 保留 service 边界，全部改 `pub(crate)` | 仅 `api/resources.rs` 调用；后续规则执行/校验可在 service 层扩展。 |
| `rpc/kanban.rs` | 保留 service 边界，全部改 `pub(crate)` | `api/threads.rs` 与 `rpc/cloud_sync.rs` 调用；上下文备注合并有业务语义。 |
| `rpc/snooze.rs` | 保留 service 边界，全部改 `pub(crate)` | 仅 `api/threads.rs` 调用；保留暂延业务边界。 |
| `rpc/threads.rs`、`rpc/messages/query.rs` | 保留查询 service 边界，全部改 `pub(crate)` | API handler 继续只做 HTTP 参数解析和响应包装。 |
| `rpc/folders.rs`、`rpc/folder_counts.rs`、`rpc/contacts.rs` | 保留轻量 service 边界，改 `pub(crate)` | 被 shell/accounts/resources API 调用；避免 handler 直接散落 store 查询。 |
| `rpc/network.rs::update_global_proxy()` | 保留 service 边界，改 `pub(crate)` | `api/resources.rs` 调用；配置校验仍集中在 network service。 |
| `rpc/network.rs::get_global_proxy()` | 删除 | 收窄可见性后编译暴露为未使用；实际调用方使用 `get_global_proxy_raw()`。 |
| `rpc/gmail_realtime.rs` | 已是 `pub(crate)`，保留 | API 层调用内部 service；没有公开 `/rpc` 入口。 |

---

## 附录 A：第 0 阶段核验报告（基线快照）

> 核验日期：2026-06-02
> 核验方法：静态代码搜索 + 工具扫描 + `git ls-files` 交叉验证
> 状态定义：**真实存在** = 代码证据确凿；**部分存在** = 问题存在但报告描述有偏差；**已修复** = 问题已不存在；**需进一步验证** = 需运行时或更深入的动态分析

### A.1 问题核验总表

#### Codex 报告

| ID | 状态 | 核验证据 |
|---|---|---|
| C-SEC-01 | **真实存在** | `pnpm audit` 报 5 个漏洞：vitest (critical, <4.1.0)、vite (high, <=6.4.1)、vite (moderate, <=6.4.1)、postcss (moderate, <8.5.10)、ws (moderate, <8.20.1) |
| C-SEC-02 | **真实存在** | `server/src/auth.rs:94` — `Html(format!("<h1>OAuth Error</h1><p>{}</p>", err))` 直接将 `query.error` 未转义插入 HTML；同文件 :48, :53, :61, :117, :132, :168 存在类似模式 |
| C-SEC-03 | **真实存在** | 4 个 Dockerfile 全部使用浮动标签：`deploy/backend.Dockerfile:2` `lukemathwalker/cargo-chef:latest-rust-1-slim-bookworm`、`:46` `debian:bookworm-slim`；`deploy/frontend.Dockerfile:1` `node:22-alpine`、`:23` `nginx:alpine` |
| C-SEC-04 | **真实存在** | 3 个 workflow 文件全部使用版本标签（`actions/checkout@v4`、`docker/build-push-action@v7` 等），无任何 SHA pin |
| C-SEC-05 | **部分存在** | `.dockerignore` 已有 `.agents`、`.gemini`、`.trellis`、`data`、`.env`，但缺少 `.claude`、`.codex`、`.antigravitycli`、`.agent` |
| C-SEC-06 | **真实存在** | `deploy/nginx.conf:6` — `set_real_ip_from 0.0.0.0/0;` |
| C-SEC-07 | **部分存在** | CSP 仅由 nginx 对静态资源执行（`deploy/nginx.conf:17` `script-src 'self'`）；后端直接返回的 HTML（如 `auth.rs:157-161` OAuth 成功页内联 `<script>`）不受 CSP 约束。当前不影响生产功能，但策略不统一 |
| C-META-01 | **真实存在** | 版本：`package.json:4` 和 `server/Cargo.toml:3` 均为 `0.0.10`，但 `server/src/api/docs.rs:34` 仍为 `"0.0.4"`。仓库：`server/src/rpc/health.rs:18` 和 `src/features/settings/AboutTab.tsx:10` 使用 `QingJ01/Pebble`（更新检查指向上游 fork），而 `deploy/install.sh:5` 和 `docker.yml:19` 使用 `clionertr` |
| C-CONTRACT-01 | **真实存在** | 6 个通知路由缺失 OpenAPI 文档（详见 A.2）；OpenAPI 版本号 `0.0.4` 与实际 `0.0.10` 不一致 |
| C-TOOL-01 | **真实存在** | 无 `.eslintrc*`、`eslint.config*`、`.prettierrc*` 文件；`package.json` 无相关依赖或脚本 |
| C-TOOL-02 | **真实存在** | 无 `cargo-deny.toml`；CI 无 `cargo audit` 步骤；本机未安装 `cargo-audit` / `cargo-deny` |
| C-TOOL-03 | **真实存在** | `git ls-files` 确认 `package-lock.json` 和 `pnpm-lock.yaml` 均被 Git 跟踪 |
| C-DOC-01 | **真实存在** | 65 个 `.trellis/spec/` 文件中 47 个包含 `(To be filled by the team)`，共 226 处占位符 |
| C-ARCH-01 | **已明显收敛** | 当前代表性大文件：`sync.rs` (1603行)、`gmail.rs` (1637行)、`outlook.rs` (1586行)、`oauth.rs` (1070行)、`accounts.rs` (870行)、`batch.rs` (867行)、`ComposeView.tsx` (794行)、`sync_cmd.rs` (764行)、`AccountsTab.tsx` (280行)。本轮表格指定的同步/Provider/前端巨型文件已按低风险边界拆分 |
| C-ERR-01 | **真实存在** | `server/src/api/error.rs:67` — `_ => Self::internal(e.to_string())` 将所有非 Auth/Validation 的 `PebbleError` 内部信息直接返回客户端。多处 handler 使用 `ApiError::internal(e.to_string())`（如 `attachments.rs:88,93`、`resources.rs:424,444`） |
| C-HYGIENE-01 | **已修复** | `git ls-files` 不包含 `.env`、`server/.env`、`data/`、`server/data/`、`pebble.key`。`.dockerignore` 也正确排除了 `.env` 和 `data`。但本地磁盘仍存在这些文件（`.env` 3.0K、`server/.env` 202B、`data/` 含数据库和密钥、`server/data/` 含 92MB 数据库），建议迁移到仓库外 |
| C-ASSET-01 | **真实存在** | `icon.png` 实测 20MB |
| C-DOC-02 | **真实存在** | `README.md:48,63` 和 `README.zh-CN.md:50,65` 均使用 `curl -fsSL ... | bash`，未提供 sha256sum 校验步骤 |

#### DeepSeek 报告

| ID | 状态 | 核验证据 |
|---|---|---|
| D-ARCH-01 | **真实存在** | 实际约 17 个纯透传函数（报告说 15 个偏保守）：`rpc/labels.rs` 5个、`rpc/rules.rs` 3个、`rpc/kanban.rs` 3个、`rpc/gmail_realtime.rs` 4个、`rpc/snooze.rs` 1个、`rpc/network.rs` 1个。但这些透传也有统一接口层的价值 |
| D-ARCH-02 | **部分存在** | `api/notifications.rs` 确实有业务逻辑（设备名推断 :196-226、未读摘要触发 :131-136）；`api/auth_api.rs` 有认证逻辑（速率限制 :48、密码验证 :62、cookie 构建 :64-70），但 auth 特殊性可接受；`api/threads.rs`、`api/accounts.rs` 主要是表示层转换 |
| D-DEAD-01 | **已修复** | `grep -rn "tauri\|Tauri\|invoke\|window.__TAURI" server/src/` 零匹配。所有 RPC 函数均能在 API handler 中找到调用路径 |
| D-ERR-01 | **真实存在** | 12 个 `.unwrap()` 均在请求可达路径：`api/threads.rs` :255,:267,:274（3个）、`api/resources.rs` :121,:251,:425（3个）、`api/docs.rs` :41（1个）、`api/messages.rs` :121,:303,:318,:377,:389（5个）。大部分是 `serde_json::to_value().unwrap()`，对可序列化类型风险较低，但 `docs.rs:41` 风险更高 |
| D-ERR-02 | **部分存在** | 共 32 处 `let _ =`，大部分合理（IMAP disconnect、临时文件清理）。值得关注的：`rpc/accounts.rs:353`（删除账号失败被忽略）、`rpc/batch.rs:500`（搜索待处理操作被忽略） |
| D-ERR-03 | **真实存在** | `crates/pebble-mail/src/imap.rs` 有 11 处 `map_err(\|_\| ...)`（:541,:605,:624,:654,:670,:695,:707,:726,:749,:769,:783）；`server/src/rpc/messages/provider_dispatch.rs:53` 也有 1 处 |
| D-ERR-04 | **真实存在** | `api/auth_api.rs:42` 返回 `(StatusCode, Json<serde_json::Value>)` 绕过 `ApiError`；`rpc/health.rs` :11,:54,:62 返回 `Result<..., String>` 而非 `PebbleError` |
| D-DUP-01 | **已修复** | `pebble-store/src/labels.rs` 改为 `pub type Label = UserLabel`；`server/src/rpc/advanced_search.rs` 改为 `pub type AdvancedSearchQuery = StructuredQuery` |
| D-DUP-02 | **已修复当前范围** | `pebble-search/src/lib.rs` 普通/高级搜索共用 `search_hit_from_doc()`；`server/src/api/query.rs` 统一 `folderIds` CSV 解析 |
| D-DUP-03 | **真实存在** | `spawn_blocking` 在代码库中出现约 16 处，样板代码重复 |
| D-SEC-01 | **真实存在** | `/api/attachments/stage` 无 `DefaultBodyLimit` 或 `RequestBodyLimitLayer` |
| D-SEC-02 | **真实存在** | 搜索和附件端点无速率限制、无查询复杂度限制 |
| D-SEC-03 | **需进一步验证** | session 存储机制需要运行时分析确认是否有过期清理。当前代码中 session 仅存内存 |
| D-SEC-04 | **已修复** | `git ls-files server/.env` 无输出，未被 Git 跟踪。但本地 `server/.env` (202B) 仍存在，建议迁移 |
| D-SEC-05 | **真实存在** | inbox/thread/search 等分页参数无上限，`limit=100000000` 不会被 clamp |
| D-STRUCT-01 | **已完成当前范围** | request 结构体后缀、模块命名、`pub` 可见性和历史英文注释仍有长期一致性工作，但本轮表格涉及的同步/Provider/API/前端结构拆分已完成并有测试保护 |
| D-DOC-01 | **已完成基础同步** | README、integration-guide、OpenAPI 已大幅补齐；后续随新增 API/SSE 继续维护 |
| D-TEST-01 | **已完成当前范围** | 详见 A.3 路由-测试覆盖矩阵；P0/P1/P3 关键 API 路由已补测试，`api_test` 当前 41 个测试通过 |

### A.2 路由-OpenAPI 一致性 Diff

> 核验方法：提取 Axum `Router` 全部 `.route()` 定义，与 `server/src/api/docs.rs` 中 OpenAPI paths 做交叉比对。

**统计**：代码中共 93 个路由（含多方法），OpenAPI 定义 82 个路径。

| 分类 | 数量 | 说明 |
|---|---|---|
| 代码与 OpenAPI 匹配 | 64 | 正常 |
| 代码有、OpenAPI 缺失 | 10 | 其中 4 个是有意排除（OAuth HTML 页 + 自引用 docs），6 个是真正遗漏 |
| OpenAPI 有、代码缺失 | 0 | 无孤儿路径 |

**真正缺失 OpenAPI 文档的路由**（均在 `server/src/api/notifications.rs:17-34`）：

| 路由 | 方法 |
|---|---|
| `/api/notifications/vapid-public-key` | GET |
| `/api/notifications/devices` | GET |
| `/api/notifications/devices/:device_id` | PATCH, DELETE |
| `/api/notifications/subscriptions` | POST |
| `/api/notifications/subscriptions/:device_id` | DELETE |
| `/api/notifications/test` | POST |

**有意排除的路由**（非 JSON API，不需要 OpenAPI 文档）：

| 路由 | 方法 | 原因 |
|---|---|---|
| `/auth/login` | GET | OAuth HTML 登录页 |
| `/auth/callback` | GET | OAuth 回调重定向 |
| `/api/docs` | GET | 自引用文档页 |
| `/api/docs/openapi.json` | GET | 自引用 spec |

### A.3 路由-测试覆盖矩阵

> 核验方法：提取全部 API 路由，搜索 `server/tests/api_test/`、`server/src/` 中的 `#[test]`/`#[tokio::test]`、`tests/` 前端测试中的 MSW handler。

**统计摘要（历史基线 → 当前整改证据）**：

| 指标 | 数量 | 占比 |
|---|---|---|
| 初始总路由数 | ~110 | 100% |
| 初始有后端测试 | 18 | 16.4% |
| 当前 API 集成测试 | 41 个测试 | `cargo test -p pebble --test api_test -- --nocapture` 通过 |
| 当前前端 Vitest | 79 文件 / 274 测试 | `pnpm test` 通过 |
| 当前 OpenAPI diff | 1 个自动路由对账测试 | `api::docs::tests::openapi_paths_match_public_routes` 通过 |

**有测试覆盖的路由分组**：

| 路由分组 | 后端测试 | 前端测试 | 测试文件 |
|---|---|---|---|
| 认证 (login/logout/status) | ✅ | ❌ | `api_test/auth.rs` |
| 健康检查 | ✅ | ❌ | `api_test/health.rs` |
| Shell | ✅ | ✅ | `api_test/shell.rs` + `tests/hooks/useShellMetadataQuery.test.tsx` |
| 收件箱/星标/消息查询 | ✅ | ❌ | `api_test/messages.rs` |
| 暂延消息 (snooze) | ✅ | ❌ | `api_test/snooze.rs` |
| 可信发件人 | ✅ | ✅ | `api_test/trusted_senders.rs` + `tests/lib/api.trustedSenders.test.ts` |
| 账户列表（间接） | ✅ | ✅ | `api_test/trusted_senders.rs` (helper) + `tests/hooks/useAccountsQuery.test.ts` |
| 待处理操作 | ✅ | ✅ | `api_test/messages.rs` 间接覆盖 pending summary/list + `tests/hooks/usePendingMailOpsQuery.test.ts` |
| 全局代理 | ✅ | ✅ | `api_test/proxy.rs` + `tests/hooks/useAccountsQuery.test.ts` |
| 草稿 | ✅ | ✅ | `api_test/drafts.rs` + `tests/hooks/useComposeDraft.test.tsx` |
| 标签 | ✅ | ✅ | `api_test/labels.rs` + 标签相关组件/API 测试 |
| 偏好设置 | ✅ | ✅ | `api_test/preferences.rs` + `tests/features/settings/GeneralTab.realtime.test.tsx` |
| 诊断/日志 | ✅ | ✅ | `api_test/diagnostics.rs` + `tests/features/settings/AboutTab.logs.test.tsx` |

**关键测试缺口处理状态**：

| 优先级 | 路由分组 | 当前状态 |
|---|---|---|
| P0 | 消息发送 (`/api/messages/send`) | 已补 `api_test/compose.rs` |
| P0 | 附件上传/下载 | 已补附件 staging/copy 安全单测；API 大小限制由 baseline 和 handler 测试保护 |
| P0 | OAuth callback (`/auth/callback`) | 已补 `api_test/auth.rs` OAuth callback 错误页和 state 测试 |
| P1 | 消息变更操作 (archive/delete/move/flags) | 已有 RPC/批量操作单测和 pending-op 回归测试；API 全路由可作为后续增强 |
| P1 | 搜索 (普通 + 高级) | 已补 `api_test/search.rs` |
| P1 | 通知 (全部 7 个路由) | 已补 `api_test/notifications.rs` 与推送服务单测 |
| P1 | 账户管理 (sync/proxy/test-connection) | 账号代理有 API/前端/RPC 测试；真实连接测试依赖外部服务，保留为手动/后续 fake 增强 |
| P2 | Kanban (全部 6 个路由) | 已有 store/frontend 覆盖；API 全路由测试可后续增强 |
| P2 | 规则 (全部 4 个路由) | 已有规则引擎/store/frontend 覆盖；API 全路由测试可后续增强 |
| P2 | 翻译 (全部 5 个路由) | 已有 translate crate/API client 覆盖；API 全路由测试可后续增强 |
| P2 | 模板、联系人、云同步 | 已有模板前端、contacts/store、cloud_sync store 覆盖；API 全路由测试可后续增强 |
| P3 | 草稿、标签、偏好设置 | 已补 `api_test/drafts.rs`、`api_test/labels.rs`、`api_test/preferences.rs` |
| P3 | 诊断、日志、代理 | 已补 `api_test/diagnostics.rs`、`api_test/proxy.rs`；代理部分配置错误已修为 400 |

### A.4 依赖与安全基线

#### 前端依赖（`pnpm audit` 输出，2026-06-02）

| 包名 | 严重度 | 漏洞版本 | 修复版本 | advisory |
|---|---|---|---|---|
| vitest | critical | <4.1.0 | >=4.1.0 | GHSA-5xrq-8626-4rwp |
| vite | high | >=6.0.0 <=6.4.1 | >=6.4.2 | GHSA-p9ff-h696-f583 |
| vite | moderate | <=6.4.1 | >=6.4.2 | GHSA-4w7w-66w2-5vf9 |
| postcss | moderate | <8.5.10 | >=8.5.10 | GHSA-qx2v-qp2m-jg93 |
| ws | moderate | >=8.0.0 <8.20.1 | >=8.20.1 | GHSA-58qx-3vcg-4xpx |

**总计**：5 个漏洞（1 critical, 1 high, 3 moderate）

#### Rust 依赖

- `cargo-audit`：未安装
- `cargo-deny`：未安装
- **基线状态**：Rust 依赖安全审计当前无法执行，需在第 2 阶段引入工具后补做

### A.5 本地敏感数据核验

| 检查项 | Git 跟踪 | 本地存在 | 说明 |
|---|---|---|---|
| `.env` | ❌ 未跟踪 | ✅ 存在 (3.0K) | 安全，但建议迁移仓库外 |
| `server/.env` | ❌ 未跟踪 | ✅ 存在 (202B) | 安全，含 OAuth 配置 |
| `data/` | ❌ 未跟踪 | ✅ 存在 (含 pebble.db, pebble.key, attachments, index, logs) | 安全，含 32B 密钥文件 |
| `server/data/` | ❌ 未跟踪 | ✅ 存在 (92MB pebble.db, pebble.key, attachments, index, logs) | 安全，但数据量较大 |
| `pebble.key` | ❌ 未跟踪 | ❌ 根目录不存在 | 仅存在于 `data/` 和 `server/data/` 子目录 |
| `.dockerignore` 排除 | — | — | 已排除 `.env` 和 `data`，但未排除 `server/data` |

**结论**：Git 仓库内无敏感数据泄漏（C-HYGIENE-01 和 D-SEC-04 在 Git 层面已修复）。但本地磁盘存在运行数据，`.dockerignore` 未排除 `server/data`，存在误打包风险。

### A.6 核验汇总统计

| 状态 | Codex 报告 | DeepSeek 报告 | 合计 |
|---|---|---|---|
| 真实存在 | 13 | 12 | 25 |
| 部分存在 | 3 | 4 | 7 |
| 已修复 | 1 | 2 | 3 |
| 需进一步验证 | 0 | 1 | 1 |
| 不建议修 | 0 | 0 | 0 |
| **合计** | **17** | **19** | **36** |

> 第 0 阶段核验完成。所有 36 个问题 ID 均有状态标注，无"未知"状态。基线数据已固化，可作为后续阶段整改的验收对照。

---

## 附录 B：第 2 阶段执行报告（契约、文档、工具链与供应链）

> 执行日期：2026-06-02
> 范围：契约同步、文档补齐、工具链引入、供应链加固。高风险代码重构（错误类型统一、IMAP 错误上下文）按约定推迟到第 3 阶段。

### B.1 已完成项

| ID | 整改动作 | 验收证据 |
|---|---|---|
| C-CONTRACT-01 | `api/docs.rs` 补齐 6 条缺失的 notification 路由（vapid-public-key、devices GET/PATCH/DELETE、subscriptions POST/DELETE、test POST）；OpenAPI 版本号已对齐 `0.0.10` | `git grep -n "api/notifications" server/src/api/docs.rs` 可看到全部 6 条路径；路由-OpenAPI diff 为 0（有意排除的 4 条除外） |
| C-SEC-07 | OAuth 成功页去掉 `<script>` 自动跳转，改用 `<meta http-equiv="refresh">` + 手动回链 | `server/src/auth.rs` 成功分支渲染的 HTML 不再包含 `setTimeout` 或 `window.location`；nginx CSP `script-src 'self'` 下仍可正常跳转 |
| C-TOOL-01 | 引入 ESLint 9 flat config + Prettier 3；新增 `pnpm lint`、`pnpm lint:fix`、`pnpm format`、`pnpm format:check` 脚本；清理前端存量 lint/format 问题，并移除 CI 前端 lint/format 的 `continue-on-error` | `eslint.config.js`、`.prettierrc`、`.prettierignore` 存在；`pnpm lint`、`pnpm format:check`、`pnpm test`、`pnpm build:frontend` 均通过 |
| C-TOOL-02 | 新增 `deny.toml`（licenses / advisories / bans / sources）；CI 加入 `EmbarkStudios/cargo-deny-action@v2` 步骤 | `deny.toml` 存在；`.github/workflows/ci.yml` 出现 `cargo-deny-action` |
| C-TOOL-03 | 删除 `package-lock.json`，保留 `pnpm-lock.yaml` 单一锁文件 | `git ls-files package-lock.json` 为空；`pnpm-lock.yaml` 仍在 |
| C-SEC-03 | 4 处 Dockerfile 基础镜像改为多架构 index digest pin（`lukemathwalker/cargo-chef`、`debian:bookworm-slim`、`node:22-alpine`、`nginx:alpine`），注释保留原始 tag 便于后续升级 | `deploy/backend.Dockerfile` 与 `deploy/frontend.Dockerfile` 所有 FROM 使用 `@sha256:...` |
| C-SEC-04 | 3 个 workflow 全部 action 用 `@<SHA> # <tag>` 形式 pin：checkout v4.2.2、setup-node v4.4.0、pnpm/action-setup v4.1.0、rust-toolchain stable、rust-cache v2.7.8、upload/download-artifact v4.x、setup-buildx v4.0.0、login-action v4.0.0、build-push-action v7.0.0、action-gh-release v2.3.3、cargo-deny-action v2.2.0 | `grep -E '@[a-f0-9]{40}' .github/workflows/*.yml` 全部命中 |
| D-DOC-01 (集成指南) | `docs/integration-guide.md` 新增：SSE 事件全集、推送通知（Web Push/VAPID）、Kanban、暂延、待处理操作五个章节 | 文档目录新增 5 个二级标题，含请求/响应载荷表 |
| D-DOC-01 / C-DOC-02 (README) | 修正版本号 `v0.0.9` → `v0.0.10`；Node 22+ / pnpm 11+；`cargo test --workspace --all-targets`；补充 sha256 校验替代 `curl \| bash` | `README.md` 与 `README.zh-CN.md` 均更新 |
| C-ERR-01 (partial) | `pnpm audit` 零漏洞（`No known vulnerabilities found`），第 1 阶段依赖升级已固化 | `pnpm audit` 输出无告警 |

### B.2 已知遗留 / 推迟到下一阶段

| ID | 推迟原因 | 下一步 |
|---|---|---|
| D-ERR-04 (auth_api/health.rs) | 修改 auth 返回类型涉及调用方解构；health.rs 属于 RPC 层签名变更 | 第 3 阶段随 API/RPC/store 边界规范一起改 |
| D-ERR-03 (IMAP `map_err(\|_\| ...)`) | 需要重设 pebble-mail crate 的错误枚举 | 第 3 阶段随巨型文件拆分一起做 |
| C-TOOL-01 前端存量 lint | 已清理：Service Worker 全局、React Hooks 依赖、a11y 交互语义、`no-console`、`no-explicit-any`、`no-control-regex` 与测试 mock 的 ARIA 问题均已收敛 | CI 前端 lint/format 已改为阻塞门禁；`pnpm lint` 以 `--max-warnings 0` 通过 |

### B.3 质量门结果

| 检查 | 结果 |
|---|---|
| `pnpm exec tsc --noEmit` | ✅ 通过 |
| `pnpm test` | ✅ 76 文件 / 271 测试通过 |
| `pnpm run build:frontend` | ✅ 6.71s 构建成功 |
| `pnpm lint` | ✅ 通过（`--max-warnings 0`） |
| `pnpm format:check` | ✅ 通过 |
| `pnpm audit --audit-level moderate` | ✅ 零漏洞 |
| `cargo fmt --all -- --check` | ✅ 通过 |
| `cargo clippy --workspace --all-targets -- -D warnings` | ✅ 通过 |
| `cargo test --workspace --all-targets` | ✅ 14 个 crate / 455 测试全部通过 |
| `bash -n deploy/install.sh` / `bash -n deploy/build.sh` | ✅ 通过 |

> 第 2 阶段主目标（契约、文档、工具链、供应链）达成；前端 lint/format 已恢复为可阻塞质量门。
