# 一键 Docker 部署

## Goal

让其他用户可以从 GitHub 拉取一个部署脚本并执行，脚本自动生成/配置运行所需环境变量，拉取已经由 GitHub Actions 构建好的 Docker 镜像，最终在服务器上成功启动 Pebble Webmail。目标是把当前“克隆源码、手动生成 bcrypt、在服务器本地构建镜像”的流程，收敛成“执行一个命令即可部署”的流程。

## What I Already Know

* 用户希望实现一键部署：从 GitHub 拉脚本执行，服务成功启动。
* 用户已配置反向代理：`mail.closev.com -> 127.0.0.1:9191`。
* 服务器条件：`sudo` 不需要密码；如果当前用户不能直接访问 Docker daemon，安装脚本需要自动改用 sudo 执行 Docker。
* 需要先实现 GitHub Actions 自动构建镜像，并且构建要带缓存。
* 可能需要改写当前缓存相关逻辑。
* 需要更新 README，让快速启动简单易懂。
* 当前仓库已有本地 Docker 构建文件：`deploy/backend.Dockerfile`、`deploy/frontend.Dockerfile`、`docker-compose.yml`、`deploy/docker-compose.yml`。
* 当前 release workflow 已能向 GHCR 构建并推送后端/前端镜像，但默认镜像名来自 `github.repository_owner`，并且 README 仍引导用户本地 `docker compose up -d --build`。
* 当前 Docker Compose 默认前端只绑定 `127.0.0.1:8080:80`，需要适配本次验收端口 `127.0.0.1:9191`。
* 后端启动要求 `PEBBLE_PASSWORD_HASH`，不设置会直接启动失败；当前 README 要求用户自行安装 `bcrypt-cli` 生成哈希。
* Docker 部署必须设置 `PEBBLE_HOST=0.0.0.0`，否则前端容器无法访问后端。
* 同源部署时 `ALLOWED_ORIGIN` 应保持为空。
* 生产 OAuth 回调根地址应为 `OAUTH_REDIRECT_URL=https://mail.closev.com`，最终回调为 `/auth/callback`。
* 一键部署脚本和 GHCR 镜像默认使用 `clionertr/Pebble`。
* 一键部署脚本默认在当前执行目录创建 `./pebble`，内部放置 `compose.yml`、`.env` 和 `data/`。
* GitHub Actions 镜像构建使用新增的独立 Docker workflow。
* 镜像构建只在推送版本 tag（`vMAJOR.MINOR.PATCH`）时触发，不再由 `master` 推送触发。
* 一键部署脚本默认拉取 `latest` 镜像标签；`latest` 由版本 tag 构建成功后更新。
* Docker 镜像构建平台为 `linux/amd64` 和 `linux/arm64`。
* GitHub Actions 构建需要按架构复用 BuildKit/GHA 缓存，避免每次重新编译 Rust/Node 依赖。
* 一键部署脚本只检测 Docker 和 Docker Compose 是否可用，不自动安装 Docker。
* 一键部署脚本启动服务后，需要等待并验证 `http://127.0.0.1:9191` 可访问。
* GHCR 镜像应面向用户公开拉取，部署脚本不要求 `docker login ghcr.io`。
* 一键部署脚本在 `PEBBLE_PUBLIC_URL` 未设置时，需要自动探测服务器 IP 并生成 `http://<ip>:9191`。
* 一键部署脚本登录密码留空时，需要默认生成 32 位随机密码并打印给用户。

## Requirements

* GitHub Actions 自动构建并推送后端和前端镜像。
* 新增独立 Docker workflow，专门负责镜像构建和推送，不把日常部署镜像逻辑混入 release workflow。
* Docker workflow 只在版本 tag 推送后发布 `latest`、完整版本号、`major.minor` 和 `sha-*` 标签，部署脚本默认使用 `latest`。
* Docker workflow 构建并推送 `linux/amd64` 和 `linux/arm64` 多架构镜像。
* 镜像构建使用按镜像和架构拆分的可复用缓存，避免每次全量重编译 Rust/Node 依赖。
* Docker workflow 尽量使用对应架构的 GitHub-hosted runner，减少 QEMU 模拟带来的构建时间。
* 一键部署脚本从 GitHub 拉取后可直接执行。
* 部署脚本默认交互式提示用户输入登录密码；用户留空时自动生成 32 位随机密码；随后生成 bcrypt hash，并按 Docker Compose 规则写入 `.env`。
* 部署脚本同时支持自动生成随机登录密码，以及通过环境变量传入登录密码，覆盖交互输入流程。
* 部署脚本在非交互环境且未传入 `PEBBLE_PASSWORD` 时，默认生成 32 位随机密码。
* 部署脚本重复执行时，如果已有 `.env` 和 `PEBBLE_PASSWORD_HASH`，需要询问用户是否重设登录密码。
* 部署脚本默认安装目录为当前执行目录下的 `./pebble`。
* 保留现有本地源码构建用 compose 文件，新增一键部署专用 compose 模板，脚本使用远端镜像模板。
* 部署脚本交互式询问公网访问地址 / OAuth 回调根地址，并写入 `OAUTH_REDIRECT_URL`；默认值为自动探测的 `http://<服务器IP>:9191`，本次验收可覆盖为 `https://mail.closev.com`。
* 部署脚本询问用户是否现在配置 Google/Microsoft OAuth；选择是时交互写入对应 Client ID/Secret，选择否时保留空值并允许服务先启动。
* 后端二进制新增 `pebble hash-password` 命令，用于把明文登录密码转换成 bcrypt hash，部署脚本通过后端镜像调用它，不要求用户安装 Rust/Python/npm。
* `pebble hash-password` 同时支持命令参数和 stdin 输入；部署脚本使用 stdin，避免密码进入 shell history。
* 部署脚本负责生成或写入 `.env`，至少保证 `PEBBLE_PASSWORD_HASH`、`OAUTH_REDIRECT_URL` 等服务启动所需配置存在。
* 部署脚本拉取远端镜像启动服务，不要求普通用户在服务器本地构建镜像。
* 部署脚本在缺少 Docker 或 Docker Compose 时给出清楚错误提示并退出，不改系统包源或自动安装；Docker daemon 对当前用户不可达但免密 sudo 可用时，自动使用 `sudo -n docker`。
* 部署脚本执行 `docker compose up -d` 后，需要轮询本机 HTTP 入口，确认服务可访问；失败时打印容器状态和排查提示。
* 部署脚本默认按公开镜像处理；如果 GHCR 包未设为 Public，脚本应在拉取失败时提示检查镜像可见性。
* 服务对宿主机暴露 `127.0.0.1:9191`，用于用户已配置的反向代理。
* README 的快速启动部分改成更短、更清晰的一键部署说明。

## Technical Approach

* 新增 `.github/workflows/docker.yml`：只在 `vMAJOR.MINOR.PATCH` tag 推送时构建镜像；发布版本号、`major.minor`、`latest` 和 `sha-*` 标签；使用 `docker/build-push-action` 的 `type=gha` 缓存，并按镜像/架构拆分 cache scope。
* 保留现有本地源码构建 compose 文件，新增远端镜像部署模板，例如 `deploy/compose.prod.yml`；一键部署脚本把模板写入 `./pebble/compose.yml`。
* 新增部署脚本，例如 `deploy/install.sh`：
  * 检查 `docker` 和 `docker compose` 是否可用；必要时使用免密 sudo fallback。
  * 在当前执行目录创建 `./pebble`。
  * 拉取/生成 `compose.yml` 和 `.env`。
  * 自动探测公网 IP，生成默认公网地址，写入 `OAUTH_REDIRECT_URL`；允许用户通过交互或 `PEBBLE_PUBLIC_URL` 覆盖。
  * 交互设置登录密码；留空生成 32 位随机密码；同时支持环境变量传入。
  * 已有密码时询问是否重设。
  * 询问是否配置 Google/Microsoft OAuth，并按选择写入 env。
  * 执行 `docker compose pull` 和 `docker compose up -d`。
  * 等待 `http://127.0.0.1:9191` 可访问，失败时输出 `docker compose ps/logs` 提示。
* 后端新增 `pebble hash-password` CLI 子命令，支持参数和 stdin；部署脚本通过后端镜像以 stdin 方式生成 bcrypt hash，再将 `$` 转义为 `$$` 写入 `.env`。
* README 快速启动改成“一条命令拉脚本并执行”，再简短说明密码、域名、OAuth 可选项和数据目录。

## Decision (ADR-lite)

**Context**: 当前部署要求用户克隆源码、手动生成 bcrypt hash、在服务器本地构建镜像。这个流程慢，且容易卡在环境和缓存问题上。

**Decision**: 使用 GitHub Actions 预构建 GHCR 公共镜像；用户侧部署脚本只负责生成配置、拉镜像、启动 compose 和健康检查。开发者本地构建 compose 保留，用户部署模板独立新增。

**Consequences**: 用户部署更简单，CI 构建承担更多工作；GHCR 包首次生成后需要确认可见性为 Public；`latest` 只随版本 tag 更新，避免每次 `master` 推送都触发昂贵镜像构建。

## Open Questions

* None.

## Acceptance Criteria

* [ ] GitHub Actions 只在版本 tag 推送后构建并推送可部署镜像。
* [ ] 仓库包含独立的 Docker 镜像构建 workflow。
* [ ] 版本 tag 镜像推送后，部署脚本默认拉取 `latest` 标签。
* [ ] 后端和前端镜像同时支持 `linux/amd64` 和 `linux/arm64`。
* [ ] 构建任务使用 BuildKit/GitHub Actions 缓存。
* [ ] 用户可通过一条从 GitHub 拉取脚本的命令完成部署。
* [ ] 脚本默认在当前目录创建 `./pebble`，并把 compose/env/data 放在其中。
* [ ] 本地源码构建 compose 仍可保留，远端镜像部署使用独立模板。
* [ ] 部署脚本支持交互输入密码、留空自动生成 32 位随机密码、环境变量传入密码三种模式。
* [ ] 部署脚本在无 TTY 且未传入密码时自动生成并打印 32 位随机密码。
* [ ] 部署脚本未设置 `PEBBLE_PUBLIC_URL` 时自动探测 IP，生成 `http://<ip>:9191` 默认 URL。
* [ ] 缺少 Docker 或 Docker Compose 时，脚本明确报错，不执行半截部署；Docker daemon 需要权限时能使用免密 sudo fallback。
* [ ] 已部署后重复执行脚本时，会询问是否重设登录密码。
* [ ] 脚本执行完成后，`docker compose ps` 显示服务已启动。
* [ ] 宿主机 `127.0.0.1:9191` 能访问 Pebble 前端。
* [ ] 脚本会等待并验证 `http://127.0.0.1:9191` 可访问；失败时输出诊断信息。
* [ ] 拉取 GHCR 镜像不需要用户登录；若镜像不可访问，脚本提示检查包是否为 Public。
* [ ] `.env` 中包含服务启动所需配置，且脚本生成的 bcrypt 中的 `$` 已按 Docker Compose 规则处理。
* [ ] `pebble hash-password` 可以生成后端登录可用的 bcrypt hash。
* [ ] `pebble hash-password` 支持参数和 stdin 两种输入方式，脚本使用 stdin。
* [ ] 脚本会询问公网访问地址，或使用自动探测 URL 默认值，并把它写入 `OAUTH_REDIRECT_URL`。
* [ ] 脚本会询问是否配置 Google/Microsoft OAuth，并按选择写入 `.env`。
* [ ] README 快速启动对普通用户可读，不要求理解本地镜像构建细节。

## Definition of Done

* 相关脚本和 compose 文件通过 shell/YAML 基础校验。
* 至少运行一次本地部署/启动路径验证，或说明无法完整验证的原因。
* README 更新。
* Trellis 质量检查完成。

## Out of Scope

* 不在本任务内配置公网反向代理或证书；用户已完成 `mail.closev.com -> 127.0.0.1:9191`。
* 不替用户申请 Google/Microsoft OAuth 凭据；脚本只负责写入用户已经准备好的 Client ID/Secret。
* 不改变 Pebble 的邮件业务功能。

## Technical Notes

* 当前 origin：`https://github.com/clionertr/Pebble.git`；upstream：`https://github.com/QingJ01/Pebble.git`。
* 默认镜像来源：`ghcr.io/clionertr/pebble` 和 `ghcr.io/clionertr/pebble-frontend`。
* 相关规范：`.trellis/spec/pebble/backend/configuration.md`、`.trellis/spec/pebble/backend/webmail-api-contracts.md`。
* `deploy/nginx.conf` 代理 `/api|events|auth|webhook` 到后端容器 `backend:3000`，符合当前 Webmail API 契约。
* 当前 `.dockerignore` 已排除 `.env`、`data`、`target`、`dist`、`.trellis` 等构建上下文噪音。
