# 设计：一键发布工作流

## 目标边界

`docker.yml` 已健全（tag → 双版本校验 → 双架构构建 → manifest + latest）。缺的是 **tag 前的自动化** 与 **tag 后的可见说明**。本次只补两个新文件 + 文档：`deploy/release.sh`（核心逻辑）、`.github/workflows/release.yml`（薄壳），并更新 README。

## 架构与数据流

```
GitHub 网页 → workflow_dispatch（填 version，可空）
      │
      ▼
release.yml（权限：contents:write；限 master）
      │ 调 deploy/release.sh
      ▼
release.sh
  ① 守卫：RELEASE_PAT 存在？分支是 master？CI 最新一次成功？
  ② 算版本：输入 || 最新 v* tag patch 递增
  ③ 校验：semver 正则（与 docker.yml 一致）· tag 未占用 · 版本递增
  ④ 改文件：package.json · server/Cargo.toml[package] · CHANGELOG（任一失败即中止）
  ⑤ git：机器人身份 commit chore(release): vX.Y.Z → 打 vX.Y.Z
  ⑥ push（RELEASE_PAT）：master 分支 + tag
      │                              │
      ▼                              ▼
   ci.yml 重新跑（质量门）      docker.yml 触发（构建镜像+latest）
  ⑦ gh release create vX.Y.Z --notes=<本次 CHANGELOG 小节>
```

## 关键设计决策

### D1：核心逻辑放脚本而非内联 YAML
`release.yml` 只负责「触发 + 调脚本 + 传参」。原因：① 版本/文件变换逻辑非平凡，放脚本可用 `--self-check`/`--dry-run` 本地验证；② 用户可本地 `./deploy/release.sh` 直接发版（同一逻辑两种入口）；③ YAML 里嵌大段 bash 无法单测。

### D2：push 用 PAT（RELEASE_PAT），不用 GITHUB_TOKEN —— 根因约束
GitHub 规则：`GITHUB_TOKEN` 引发的事件（除 `workflow_dispatch`/`repository_dispatch`）**不会**再触发新 workflow run。若发布流程用 `GITHUB_TOKEN` 推 tag，`docker.yml` 永远不跑，发布会"看似成功实则无镜像"。所以必须用仓库管理员的 PAT（classic：`repo` 权限；或 fine-grained：`contents: read+write`），存为 secret `RELEASE_PAT`。
- 脚本在任何 git 写操作前检查该 secret 是否存在，缺失则打印中文指引并退出。
- 同一 PAT 同时用于 push（git credential）与 `gh release create`（`GH_TOKEN`）。

### D3：文件变换（只动三处，均在仓库根）
| 文件 | 方式 | 说明 |
|---|---|---|
| `package.json` | `npm pkg set version=X.Y.Z` | 保序改写，不依赖 node 脚本 |
| `server/Cargo.toml` | node 单行脚本 `s/^(version\s*=\s*)"[^"]*"/$1"X.Y.Z"/m`（只替换第一处） | 命中 `[package]`，不碰同行内联依赖 |
| `CHANGELOG.md` | awk：在第一个 `### ` 小节前插入 `## [X.Y.Z] - <UTC 日期>` + 空行 | 旧条目自然归入新版本；插入失败（无 `### `）则退出报错，不静默 |

三个变换都做成「写回前先断言结果包含/不包含预期字符串」，任一不满足即中止。

### D4：版本来源与校验
- 输入留空 → `git describe --tags --match 'v*.*.*' --abbrev=0` 取最新 tag，patch +1（预发布 tag 视为其正式版的 patch）。
- 输入 → 去 `v` 前缀后过 semver 正则；与最新 tag 用 `sort -V` 比较必须递增；`git rev-parse -q --verify refs/tags/v$v` 必须失败（tag 未占用）。
- 合法版本再写入文件；`docker.yml` 自身的校验是兜底第二道。

### D5：CI 绿门
`gh run list --workflow ci.yml --branch master --limit 1 --json conclusion,status --jq` 取最新一次运行：`status == completed` 且 `conclusion == success` 才放行，否则中止。防止把坏代码发上线（`docker.yml` 只构建不跑测试，绿门必须在发布侧补上）。本地模式无 `RELEASE_PAT` 时跳过此门。

### D6：git 与 Release
- 身份：`github-actions[bot] <41898282+github-actions[bot]@users.noreply.github.com>`（不依赖真实用户配置）。
- 提交消息 `chore(release): vX.Y.Z`，annotated tag `git tag -a vX.Y.Z -m "vX.Y.Z"`。
- 推送顺序：`git push origin master` 与 `git push origin vX.Y.Z` 都要（tag 指向的 commit 必须包含版本改动；master 也必须有版本改动，否则下次自动递增读到旧版本）。
- Release notes：`awk` 抽取本次小节 → `gh release create vX.Y.Z --title vX.Y.Z --notes-file <临时文件>`。

## 脚本接口

```
deploy/release.sh [version] [--dry-run] [--self-check]
  version          默认空 → 自动 patch 递增
  --dry-run        在临时目录复制三份文件做变换并打印 diff，不碰仓库、不 git
  --self-check     用内置夹具断言三处变换的正确性（ponytail 要求的可运行检查）
  无参数           完整发布（需 RELEASE_PAT 环境变量/secret）
```

`release.yml` 传参：`${{ inputs.version }}`，并注入 `RELEASE_PAT`、仓库 owner/repo 为环境变量。

## 兼容性 / 回滚
- 发布动作全部由 `deploy/release.sh` 一个入口控制；改动只加文件、不改旧文件，`git revert` 即可整体回滚。
- 工作流错误会导致 tag 未创建或构建失败，但不会损坏已有镜像；`docker.yml` 按 tag 语义发布，`latest` 只随新 tag 更新，天然可回滚到旧 tag。

## 风险
- **PAT 权限不足**：push 失败时给出"检查 RELEASE_PAT 的 repo/contents 权限"提示。
- **并发误触发**：`concurrency` 按 ref 分组 + `cancel-in-progress`，防止连点两次。
- **CHANGELOG 无 `###` 小节**：awk 未插入即断言失败中止（避免静默丢版本条目）。
- **最新 tag 是预发布**：patch 递增基于其正式版数字，可继续发正式版。
