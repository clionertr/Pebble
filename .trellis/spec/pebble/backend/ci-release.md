# CI 与发布契约

> 范围：`.github/workflows/`、`deploy/release.sh`、版本号双文件同步、镜像分发。

## 发布管线三段

| 环节 | 工作流 / 脚本 | 触发条件 | 职责 |
|---|---|---|---|
| 质量门 | `ci.yml` | push master / PR | lint、format、clippy、测试、前端构建 |
| 镜像构建 | `docker.yml` | 推送 `v*.*.*` tag | 校验 tag==双版本号，构建 amd64+arm64，推 GHCR，非预发布更新 `latest` |
| 一键发布 | `release.yml` + `deploy/release.sh` | 网页 `workflow_dispatch` | 算版本 → 同步版本文件 → 更新 CHANGELOG → commit + tag + push → 触发上面两段 → 创建 GitHub Release |

用户端部署保持 `deploy/install.sh` 一条命令拉 `latest`，不随发布改动。

## 硬性约束

- **tag 推送必须用 PAT（`RELEASE_PAT`），不能用 `GITHUB_TOKEN`。**
  原因：GitHub 规定 `GITHUB_TOKEN` 引发的事件（除 `workflow_dispatch`/`repository_dispatch`）不会再次触发 workflow run；用 `GITHUB_TOKEN` 推 tag，`docker.yml` 永远不会执行，发布会"看似成功实则无镜像"。PAT 需要仓库 `contents` 读写权限，配置在 Settings → Secrets and variables → Actions。
- **版本号只有两个源头**：`package.json` 的 `version` 和 `server/Cargo.toml` 的 `[package]` version，二者必须等于发布 tag（去掉 `v`）。`docker.yml` 会在构建前二次校验。
- **消费版本号的地方必须构建时派生，不得硬编码**：
  - 前端 About 页：`import pkg from "../../../package.json"` 读 `pkg.version`（配合 tsconfig `resolveJsonModule`）。
  - 后端 OpenAPI（`server/src/api/docs.rs`）：`env!("CARGO_PKG_VERSION")`。
  派生处自动跟随发布，`release.sh` 无需改它们；重新硬编码会在下个版本漂移。
- **Cargo.toml 只改 `[package]` 版本**：用正则 `^(version\s*=\s*)"[^"]*"` 只替换第一处行首 version；本项目依赖均为同行内联 `{ version = ... }`，不会被命中。若未来有行首裸 `version =` 的依赖，正则需收紧。内部 workspace crate（`crates/pebble-*`，版本 `0.1.0`）不随应用发布版本同步，属有意为之。
- **发布前绿门**：master 最新一次 `ci.yml` 必须 `success` 才允许发布（`docker.yml` 只构建不跑测试，绿门必须在发布侧补上）。
- **CHANGELOG**：keep-a-changelog；发布把 `## [Unreleased]` 下条目归入新的 `## [X.Y.Z] - <UTC日期>`，`## [Unreleased]` 置顶留空；插入失败必须中止，不得静默跳过。

## 发布入口

`deploy/release.sh` 是唯一发布入口（workflow 只是薄壳调用它）：

```bash
./deploy/release.sh                 # 完整发布，版本号自动 patch 递增
./deploy/release.sh v0.0.13         # 完整发布，指定版本
./deploy/release.sh --dry-run v0.0.13  # 演练：临时目录演示文件 diff，不碰仓库
./deploy/release.sh --self-check    # 自检文件变换逻辑
```

完整发布需 `RELEASE_PAT`；缺少密钥、版本非法、tag 已存在、版本不递增、CI 非绿均非零退出并报错。

## 常见错误

- 手工改版本号只改了一个源头文件 → `docker.yml` 构建失败；永远用 `deploy/release.sh` 或 `--dry-run` 预演。
- 在 About 页或 OpenAPI 里硬编码版本字符串 → 下个版本必漂移；改用构建时派生（见上）。
- 新增依赖把 `version =` 写成行首裸行 → 版本同步正则可能误改依赖；保持依赖内联写法。
- 误删 `## [Unreleased]` 标题 → 下次发布 CHANGELOG 插入断言失败。
