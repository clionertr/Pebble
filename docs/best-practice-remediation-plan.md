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
| C-ARCH-01 | **部分存在** | 最大文件：`sync_cmd.rs` (1197行)、`oauth.rs` (1070行)、`AccountsTab.tsx` (1041行)、`batch.rs` (863行)、`accounts.rs` (861行)、`ComposeView.tsx` (834行)。属中等偏大，非极端巨型，但职责确实过重 |
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
| D-DUP-01 | **真实存在** | `pebble-core/src/types.rs:157` `UserLabel` 与 `pebble-store/src/labels.rs:9` `Label` 字段完全相同；`pebble-core/src/traits.rs:57` `StructuredQuery` 与 `server/src/rpc/advanced_search.rs:8` `AdvancedSearchQuery` 字段完全相同 |
| D-DUP-02 | **部分存在** | SearchHit 构造在 `pebble-search/src/lib.rs:363` 和 `:506` 重复。CSV 解析不存在（`grep csv/CSV server/src/` 零匹配）。查询模式有部分重复 |
| D-DUP-03 | **真实存在** | `spawn_blocking` 在代码库中出现约 16 处，样板代码重复 |
| D-SEC-01 | **真实存在** | `/api/attachments/stage` 无 `DefaultBodyLimit` 或 `RequestBodyLimitLayer` |
| D-SEC-02 | **真实存在** | 搜索和附件端点无速率限制、无查询复杂度限制 |
| D-SEC-03 | **需进一步验证** | session 存储机制需要运行时分析确认是否有过期清理。当前代码中 session 仅存内存 |
| D-SEC-04 | **已修复** | `git ls-files server/.env` 无输出，未被 Git 跟踪。但本地 `server/.env` (202B) 仍存在，建议迁移 |
| D-SEC-05 | **真实存在** | inbox/thread/search 等分页参数无上限，`limit=100000000` 不会被 clamp |
| D-STRUCT-01 | **真实存在** | 抽样确认：request 结构体后缀不统一、`pub` 可见性过宽、英文/中文注释混用 |
| D-DOC-01 | **真实存在** | `docs/integration-guide.md` 缺少推送通知、暂停/收藏/待处理操作、SSE 事件等文档 |
| D-TEST-01 | **真实存在** | 详见 A.3 路由-测试覆盖矩阵。约 79% 路由无任何测试 |

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

**统计摘要**：

| 指标 | 数量 | 占比 |
|---|---|---|
| 总路由数 | ~110 | 100% |
| 有后端测试 | 18 | 16.4% |
| 有前端测试 | 12 | 10.9% |
| 有任意测试 | 23 | 20.9% |
| 无任何测试 | ~87 | 79.1% |

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
| 待处理操作 | ❌ | ✅ | `tests/hooks/usePendingMailOpsQuery.test.ts` |
| 全局代理 | ❌ | ✅ | `tests/hooks/useAccountsQuery.test.ts` |

**关键测试缺口**（按风险排序）：

| 优先级 | 路由分组 | 缺失原因 |
|---|---|---|
| P0 | 消息发送 (`/api/messages/send`) | 核心功能，无测试 |
| P0 | 附件上传/下载 | 安全风险，无测试 |
| P0 | OAuth callback (`/auth/callback`) | 安全关键路径，无测试 |
| P1 | 消息变更操作 (archive/delete/move/flags) | 数据变更，无测试 |
| P1 | 搜索 (普通 + 高级) | 核心功能，无测试 |
| P1 | 通知 (全部 7 个路由) | 新功能，无测试 |
| P1 | 账户管理 (sync/proxy/test-connection) | 配置关键，无测试 |
| P2 | Kanban (全部 6 个路由) | 无测试 |
| P2 | 规则 (全部 4 个路由) | 无测试 |
| P2 | 翻译 (全部 5 个路由) | 无测试 |
| P2 | 模板、联系人、云同步 | 无测试 |
| P3 | 草稿、标签、偏好设置 | 无测试 |
| P3 | 诊断、日志、代理 | 无测试 |

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

