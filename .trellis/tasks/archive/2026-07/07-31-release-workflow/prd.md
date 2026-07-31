# 一键发布工作流（release.yml）

## Goal

让"上线发版"从手工多步操作收敛为"在 GitHub 网页填一个版本号、点一下按钮"。点完后自动完成：版本号同步（`package.json` + `server/Cargo.toml`）、CHANGELOG 更新、打 `vX.Y.Z` tag、触发 `docker.yml` 构建镜像、生成 GitHub Release 说明。解决当前三个痛点：手工改双版本号容易漏、网页上无发布入口、Releases 页面是空的。

用户端"一条命令部署"（`deploy/install.sh`）保持不动。

## What I Already Know

* 发布链路现状：`ci.yml`（质量门，push master / PR 时跑）→ `docker.yml`（仅 `v*.*.*` tag 推送时构建 amd64+arm64 镜像推 GHCR，并校验 tag 必须同时等于 `package.json` 与 `server/Cargo.toml` 版本）→ 用户端 `install.sh` 一条命令部署。
* `docker.yml` 的 tag 正则：`^v(\d+\.\d+\.\d+(-[0-9A-Za-z][0-9A-Za-z.-]*)?)$`，即 `v0.0.13` 或 `v0.0.13-1` 形式；文件里写入的是去掉 `v` 的版本号。
* `server/Cargo.toml` 只有 `[package]` 区（第 3 行 `version = "0.0.12"`）是行首裸 version，依赖均为同行内联，正则 `^version\s*=\s*"..."` 只命中一处，可安全替换。
* `CHANGELOG.md` 遵循 Keep a Changelog，顶部结构为 `## [Unreleased]` 空标题 + `### Added/Changed/...` 小节；发布时应把旧小节归入新版本，`## [Unreleased]` 置顶留空待下一轮。
* 当前版本号：`package.json` 与 `server/Cargo.toml` 均为 `0.0.12`；已打 tag 到 `v0.0.12`。
* 当前 GitHub Releases 页面为空（打过 tag 但从未创建 Release）。
* 关键约束：**`GITHUB_TOKEN` 推送的 tag 不会再次触发 `docker.yml`**（GitHub 规则，除 `workflow_dispatch`/`repository_dispatch` 外的 GITHUB_TOKEN 事件不再触发工作流）。因此发布工作流推 tag 必须用仓库管理员持有的 PAT（存为 `RELEASE_PAT` secret），否则镜像永远不构建。
* `latest` tag 的更新已由 `docker.yml` 的 manifest job 负责（非预发布版本自动打 `latest`），本次不动它。
* runner 自带 `gh` CLI 和 `node`、`npm`、`git`。
* 上游有 `upstream` remote（`QingJ01/Pebble.git`），发布只应作用于 `origin`（`clionertr/Pebble.git`）。

## Requirements

* 新增 `.github/workflows/release.yml`，触发方式仅 `workflow_dispatch`（网页手动触发），只允许在 `master` 分支上执行。
* 网页输入：`version`（可空）。留空时自动读取最新 `v*` tag 做 patch 递增（如 `0.0.12` → `0.0.13`）；填写时必须通过 semver 校验（与 `docker.yml` 正则一致），且新 tag 不得已存在、不得早于/等于当前最新 tag。
* 发布前校验：最新一次 `ci.yml` 在 master 上的结果为成功，否则中止（防止把坏代码发上线）。
* 版本同步：`package.json` 与 `server/Cargo.toml` 的 `[package]` 版本同时改为新版本；只改 Cargo.toml 的包版本，不动任何依赖版本。
* CHANGELOG：把当前 `## [Unreleased]` 下的旧条目归入新的 `## [X.Y.Z] - <UTC日期>` 小节，`## [Unreleased]` 置顶留空；若未插入新标题则视为失败（宁可中止不可静默跳过）。
* Git 操作：以机器人身份（`github-actions[bot]`）提交 `chore(release): vX.Y.Z`，打 `vX.Y.Z` 标签，同时推送 master 分支与标签；推送使用 `RELEASE_PAT`。
* 推 tag 后自动触发 `docker.yml` 构建镜像（依赖 `RELEASE_PAT`）；本次不修改 `docker.yml` 构建逻辑。
* 创建 GitHub Release：版本说明取自刚写入的 CHANGELOG 小节，Release 名与 tag 一致。
* 失败必须大声：缺少 `RELEASE_PAT`、版本非法、tag 已存在、版本不递增、CI 非绿、文件未成功修改，任一情况非零退出并给出明确中文错误信息。
* 核心逻辑抽成 `deploy/release.sh`（可在本地运行，`--dry-run` 只在临时目录演练文件编辑不提交不推送；`--self-check` 自带断言校验文件变换逻辑）。workflow 只是薄壳调用该脚本。
* README（英文 + 中文）补充"如何一键发版"：网页步骤 + 首次使用前创建 PAT 并配 `RELEASE_PAT` secret 的一次性说明。

## Constraints

* 用户端部署流程（`install.sh` / `compose.prod.yml`）保持不变。
* 不引入 release-please 等第三方自动化工具。
* 不修改 `docker.yml` 的镜像构建与校验逻辑（它已健全，只是没人自动打 tag）。
* 发布脚本不得误改 `server/Cargo.toml` 依赖区、不得破坏 CHANGELOG 其余历史。

## Acceptance Criteria

* [ ] `.github/workflows/release.yml` 存在，仅 `workflow_dispatch` 触发，并校验分支为 master。
* [ ] 在 GitHub 网页运行该工作流（填 `v0.0.13` 或留空），master 上生成一个提交 `chore(release): v0.0.13`，其中 `package.json`、`server/Cargo.toml` 版本均为 `0.0.13`。
* [ ] `CHANGELOG.md` 出现 `## [0.0.13] - <日期>`，原 `## [Unreleased]` 下的条目归入该小节，`## [Unreleased]` 保留在顶部且为空。
* [ ] 生成的 `v0.0.13` 标签推送后能触发 `docker.yml` 构建（文档写明依赖 `RELEASE_PAT`，缺 secret 时脚本明确报错）。
* [ ] GitHub Releases 页面出现 `v0.0.13` Release，说明文本为本次 CHANGELOG 小节内容。
* [ ] 留空版本号时自动按 patch 递增（由脚本 `--dry-run` 验证）。
* [ ] `deploy/release.sh --self-check` 通过；`--dry-run vX.Y.Z` 在临时目录生成正确文件差异且不触碰真实仓库。
* [ ] 缺少 `RELEASE_PAT` / 版本非法 / tag 已存在 / 版本不递增 / CI 非绿 时，脚本或工作流以非零状态退出并给出明确错误信息。
* [ ] `ci.yml` 在发布提交推送后仍为绿色（发布提交本身过质量门）。
* [ ] README（EN + zh-CN）有"一键发版 + PAT 配置"章节。

## Definition of Done

* `deploy/release.sh` 通过 `bash -n` 与 `--self-check`，`--dry-run` 输出符合预期。
* `release.yml` 通过 YAML 语法校验。
* 本地完成一次"发布演练"：用 `--dry-run v0.0.99` 验证文件变换结果与 git 操作序列。
* README 双语更新。
* Trellis 质量检查完成。

## Out of Scope

* 不为用户创建 PAT / secret（用户需自己在 GitHub 网页配置一次 `RELEASE_PAT`）。
* 不修改 `docker.yml` 镜像构建、校验、multi-arch manifest 逻辑。
* 不修改用户端 `install.sh`、`compose.prod.yml`、`.env` 模板。
* 不做 release-please 全自动版本推导。
* 不在本次实施过程中真实触发一次发布（避免产生真实 tag 与镜像）。

## Technical Notes

* 关联任务：`05-19-one-click-docker-deploy`（现有 docker.yml 与 install.sh 的来源）。
* 相关文件：`.github/workflows/ci.yml`、`.github/workflows/docker.yml`、`deploy/install.sh`、`deploy/compose.prod.yml`、`package.json`、`server/Cargo.toml`、`CHANGELOG.md`、`README.md`、`README.zh-CN.md`。
* GitHub 官方文档要点：使用 `GITHUB_TOKEN` 推送 tag 不会触发新的 workflow run（"Recursive workflows"限制），发布流程需 PAT。
