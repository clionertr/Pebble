# 实施：一键发布工作流

## 执行清单（按序）

- [x] 1. 编写 `deploy/release.sh`
  - [x] 1.1 `set -euo pipefail`，函数化：`die`/`log`/`next_patch`/`validate_version`/`edit_files`/`ci_gate`/`git_ops`
  - [x] 1.2 模式分发：`--self-check` / `--dry-run` / 完整发布
  - [x] 1.3 文件变换（D3 表格三处），每处变换后断言
  - [x] 1.4 版本计算与校验（D4）
  - [x] 1.5 CI 绿门（D5，本地模式跳过）
  - [x] 1.6 git 提交 + 打 tag + 双推送（D6，用 RELEASE_PAT）
  - [x] 1.7 `gh release create`（D6，notes 取本次 CHANGELOG 小节）
- [x] 2. 本地验证脚本
  - [x] 2.1 `bash -n deploy/release.sh`
  - [x] 2.2 `deploy/release.sh --self-check` 通过
  - [x] 2.3 `deploy/release.sh --dry-run v0.0.99`：检查输出的 diff 中三处文件均正确、临时目录已清理、仓库文件未被改动（`git status` 干净）
- [x] 3. 编写 `.github/workflows/release.yml`
  - [x] 3.1 `workflow_dispatch` + `inputs.version`（可空）；`concurrency` 防连点
  - [x] 3.2 `permissions: contents: write`
  - [x] 3.3 调 `deploy/release.sh`，注入 `RELEASE_PAT`、owner/repo 环境变量；缺失 secret 时由脚本报错退出
- [x] 4. YAML 语法校验（python yaml 解析 `release.yml`，actionlint 全量通过）
- [x] 5. 更新文档
  - [x] 5.1 `README.md` + `README.zh-CN.md` 增补「一键发版」章节：网页操作步骤 + PAT 创建与 `RELEASE_PAT` 配置一次性说明
- [x] 6. 质量检查
  - [x] 6.1 复核：脚本不误改 Cargo 依赖 / 不破坏 CHANGELOG 历史 / `docker.yml` 未被触碰
  - [x] 6.2 跑 Trellis 检查（trellis-check 发现 require_pat 的 `&& die` set -e 静默退出致命 bug，已修复；另修复 ci_gate 错误 JSON 污染、预发布提升误拦、分支守卫、CHANGELOG 重复插入）
- [x] 7. 提交（本任务只提交实现，不触发真实发布；提交信息 `feat(ci): 一键发布工作流`）

## 验证命令

```bash
bash -n deploy/release.sh
deploy/release.sh --self-check
deploy/release.sh --dry-run v0.0.99     # 期望：打印三处 diff，仓库 git status 干净
git status                               # 期望：clean（dry-run 不落盘）
python3 - <<'PY'
import yaml; yaml.safe_load(open('.github/workflows/release.yml')); print('yaml ok')
PY
```

全部通过 ✅（含 actionlint 校验 `.github/workflows/*.yml` exit 0）。

## 评审门

- 脚本与 YAML 通过上述命令后，进入 6.2 Trellis 检查。✅
- 任何一步失败回到对应清单项，不回退到内联 YAML 方案（D1 已定）。
