#!/usr/bin/env bash
# 一键发布：同步版本号 → 更新 CHANGELOG → 打 tag → 推送（触发 docker.yml 构建镜像）→ 创建 GitHub Release。
#
# 用法：
#   ./deploy/release.sh                 # 完整发布：版本号自动 patch 递增（0.0.12 → 0.0.13）
#   ./deploy/release.sh v0.0.13         # 完整发布：指定版本（或 0.0.13）
#   ./deploy/release.sh --dry-run v0.0.13   # 演练：在临时目录演示文件变更，不提交不推送
#   ./deploy/release.sh --self-check    # 自检：断言三处文件变换逻辑正确
#
# 完整发布需要环境变量 RELEASE_PAT（仓库管理员 PAT，具备 contents 读写权限）。
# 为什么不能用 GITHUB_TOKEN：GitHub 规定 GITHUB_TOKEN 引发的事件不会再触发新的
# workflow run，用它推 tag 的话 docker.yml 永远不会执行。

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
REPO="${REPO:-clionertr/Pebble}"
TMPCLEAN=""
trap '[[ -n "$TMPCLEAN" ]] && rm -rf "$TMPCLEAN"' EXIT

log()  { printf "\033[1;34m==>\033[0m %s\n" "$*" >&2; }
warn() { printf "\033[1;33mWarning:\033[0m %s\n" "$*" >&2; }
die()  { printf "\033[1;31mError:\033[0m %s\n" "$*" >&2; exit 1; }

# ---------- 版本计算与校验 ----------

latest_tag() {
  git tag --list 'v*' --sort=-v:refname 2>/dev/null | head -n 1
}

# 由 v0.0.12 / v0.0.12-1 算出下一个 patch 版本
next_patch() {
  local core="${1#v}"
  core="${core%%-*}"
  local major minor patch
  IFS=. read -r major minor patch <<<"$core"
  printf "%s.%s.%d" "$major" "$minor" "$((patch + 1))"
}

# 归一化：去掉可选 v 前缀，且必须符合 docker.yml 的 semver 正则
normalize_ver() {
  local re='^v?([0-9]+\.[0-9]+\.[0-9]+(-[0-9A-Za-z][0-9A-Za-z.-]*)?)$'
  if [[ "$1" =~ $re ]]; then
    printf "%s" "${BASH_REMATCH[1]}"
  else
    die "版本号 '$1' 非法，应为 vX.Y.Z 或 X.Y.Z（可选 -prerelease），与 docker.yml 校验一致"
  fi
}

resolve_version() {
  local input="${1:-}"
  if [[ -n "$input" ]]; then printf "%s" "$input"; return 0; fi
  local latest
  latest="$(latest_tag)"
  if [[ -z "$latest" ]]; then printf "0.0.1"; else next_patch "$latest"; fi
}

validate_release() { # ver
  local ver="$1" latest
  latest="$(latest_tag)"
  if git rev-parse -q --verify "refs/tags/v${ver}" >/dev/null 2>&1; then
    die "tag v${ver} 已存在，请换一个版本号"
  fi
  if [[ -n "$latest" ]]; then
    local latest_nover="${latest#v}"
    if [[ "$latest_nover" == "$ver" ]]; then
      die "版本 $ver 已是当前最新版本（$latest）"
    fi
    local higher hcore
    higher="$(printf '%s\n%s\n' "$latest_nover" "$ver" | sort -V | tail -n 1)"
    if [[ "$higher" != "$ver" ]]; then
      # 特例：允许把预发布提升为同核心正式版（0.0.13-1 → 0.0.13），sort -V 会把 -1 排到正式版之后
      hcore="${higher%%-*}"
      [[ "$hcore" == "$ver" ]] || die "新版本 $ver 必须高于当前最新版本 $latest_nover"
    fi
  fi
  log "发布版本：v${ver}${latest:+（当前最新 ${latest}）}"
}

# ---------- 文件变换 ----------

edit_package_json() { # dir, ver
  local dir="$1" ver="$2"
  ( cd "$dir" && npm pkg set version="$ver" >/dev/null )
  grep -q "\"version\": \"${ver}\"" "$dir/package.json" \
    || die "package.json 版本未更新为 $ver"
}

edit_cargo_toml() { # dir, ver —— 只改第一处行首 version（即 [package]），不碰依赖
  local dir="$1" ver="$2"
  node - "$dir/server/Cargo.toml" "$ver" <<'NODE'
const file = process.argv[2];
const ver = process.argv[3];
const fs = require('node:fs');
let s = fs.readFileSync(file, 'utf8');
const re = /^(version\s*=\s*)"[^"]*"/m;
if (!re.test(s)) { console.error('server/Cargo.toml 中未找到可替换的 version 行'); process.exit(1); }
fs.writeFileSync(file, s.replace(re, `$1"${ver}"`));
NODE
  grep -qE "^version[[:space:]]*=[[:space:]]*\"${ver}\"" "$dir/server/Cargo.toml" \
    || die "server/Cargo.toml 包版本未更新为 $ver"
}

edit_changelog() { # dir, ver, date
  local dir="$1" ver="$2" date="$3"
  local f="$dir/CHANGELOG.md"
  local inserted=0
  if grep -q "^## \[${ver}\] -" "$f"; then
    die "CHANGELOG 已存在 ${ver} 小节，可能此前已部分发布，请人工检查后再发"
  fi
  if grep -q '^### ' "$f"; then
    # 有小节：把新标题插到第一个 ### 之前，旧条目自然归入新版本
    awk -v ver="$ver" -v date="$date" '
      /^### / && !done { print "## [" ver "] - " date; print ""; done=1 }
      { print }
    ' "$f" > "$f.tmp"
  else
    # 无小节（Unreleased 为空）：插到 ## [Unreleased] 之后
    awk -v ver="$ver" -v date="$date" '
      /^## \[Unreleased\]$/ && !done {
        print; print ""; print "## [" ver "] - " date; done=1; next
      }
      { print }
    ' "$f" > "$f.tmp"
  fi
  grep -q "^## \[${ver}\] - ${date}" "$f.tmp" && inserted=1
  if [[ "$inserted" != "1" ]]; then rm -f "$f.tmp"; die "CHANGELOG 未插入 ${ver} 版本标题，中止"; fi
  mv "$f.tmp" "$f"
}

edit_files() { # dir, ver —— 幂等，可对临时目录演练
  local dir="$1" ver="$2"
  local date; date="$(date -u +%F)"
  edit_package_json "$dir" "$ver"
  edit_cargo_toml "$dir" "$ver"
  edit_changelog "$dir" "$ver" "$date"
  log "已同步版本 $ver（package.json / server/Cargo.toml / CHANGELOG）"
}

# ---------- 完整发布的门禁与动作 ----------

guard_branch() {
  local cur
  cur="$(git branch --show-current 2>/dev/null || true)"
  # workflow 里 checkout 是 detached HEAD（cur 为空），由 release.yml 的 GITHUB_REF 守卫兜底；本地必须显式在 master
  if [[ -n "$cur" && "$cur" != "master" ]]; then
    die "发布只允许在 master 分支执行（当前在 $cur 分支）"
  fi
}

require_pat() {
  if [[ -z "${RELEASE_PAT:-}" ]]; then
    die "缺少 RELEASE_PAT 密钥。GITHUB_TOKEN 推送的 tag 不会触发 docker.yml（GitHub 递归工作流限制）。请在仓库 Settings → Secrets and variables → Actions 添加 RELEASE_PAT（PAT 需仓库 contents 读写权限）。"
  fi
}

ci_gate() {
  local ci
  if ! ci="$(GH_TOKEN="$RELEASE_PAT" gh api "repos/${REPO}/actions/workflows/ci.yml/runs?branch=master&status=completed&per_page=1" --jq '.workflow_runs[0].conclusion' 2>/dev/null)"; then
    die "无法读取 master CI 状态，请检查 RELEASE_PAT 权限或稍后重试"
  fi
  if [[ -z "$ci" ]]; then
    die "master 上暂无可用的已完成 CI 运行，请先跑一次 ci.yml 再发布"
  fi
  [[ "$ci" == "success" ]] || die "master 最新一次 CI 结果为 '$ci'，不是 success。请先修复 CI 再发布。"
  log "CI 绿门通过（master 最新一次 ci.yml = success）"
}

git_ops() { # ver
  local ver="$1"
  local url="https://x-access-token:${RELEASE_PAT}@github.com/${REPO}.git"
  cd "$ROOT"
  git config user.name  "github-actions[bot]"
  git config user.email "41898282+github-actions[bot]@users.noreply.github.com"
  git add package.json server/Cargo.toml CHANGELOG.md
  if git diff --cached --quiet; then
    warn "版本文件无变化，跳过提交（可能已发布过该版本）"
  else
    git commit -m "chore(release): v${ver}"
  fi
  git tag -a "v${ver}" -m "v${ver}"
  git push "$url" "refs/heads/master:refs/heads/master"
  git push "$url" "refs/tags/v${ver}:refs/tags/v${ver}"
  log "已推送 master 与 tag v${ver}（将触发 ci.yml 与 docker.yml）"
}

release_note() { # ver
  local ver="$1"
  TMPCLEAN="$(mktemp)"
  awk -v v="$ver" '
    $0 ~ "^## \\[" v "\\]" { f = 1; next }
    f && /^## \[/ { exit }
    f { print }
  ' "$ROOT/CHANGELOG.md" > "$TMPCLEAN"
  sed -i '1{/^$/d;}' "$TMPCLEAN"
  [[ -s "$TMPCLEAN" ]] || die "CHANGELOG 未找到 v${ver} 小节，无法生成 Release 说明"
  GH_TOKEN="$RELEASE_PAT" gh release create "v${ver}" --title "v${ver}" --notes-file "$TMPCLEAN" \
    || die "GitHub Release 创建失败，请检查 RELEASE_PAT 权限"
  log "已创建 GitHub Release v${ver}"
}

# ---------- 自检与演练 ----------

self_check() {
  local dir
  TMPCLEAN="$(mktemp -d)"
  dir="$TMPCLEAN"
  mkdir -p "$dir/server"
  cat > "$dir/package.json" <<'EOF'
{
  "name": "pebble",
  "private": true,
  "version": "9.9.9"
}
EOF
  cat > "$dir/server/Cargo.toml" <<'EOF'
[package]
name = "pebble"
version = "9.9.9"

[dependencies]
tokio = { version = "1.0", features = ["full"] }
tower-http = { version = "0.6.8", features = ["cors"] }
EOF
  cat > "$dir/CHANGELOG.md" <<'EOF'
# Changelog

## [Unreleased]

### Added

- 新功能 A

### Fixed

- 修复 B

## [0.0.1] - 2026-01-01

### Fixed

- 历史修复
EOF

  edit_package_json "$dir" "0.0.13"
  edit_cargo_toml "$dir" "0.0.13"
  edit_changelog "$dir" "0.0.13" "2026-07-31"

  grep -q '"version": "0.0.13"' "$dir/package.json" || die "self-check: package.json 未更新"
  grep -q '^version = "0.0.13"' "$dir/server/Cargo.toml" || die "self-check: Cargo.toml 未更新"
  grep -q 'tokio = { version = "1.0"' "$dir/server/Cargo.toml" || die "self-check: Cargo.toml 依赖被误改"
  grep -q 'tower-http = { version = "0.6.8"' "$dir/server/Cargo.toml" || die "self-check: Cargo.toml 依赖被误改"
  grep -q '^## \[Unreleased\]$' "$dir/CHANGELOG.md" || die "self-check: Unreleased 标题丢失"
  grep -q '^## \[0.0.13\] - 2026-07-31$' "$dir/CHANGELOG.md" || die "self-check: 新版本标题未插入"
  local i_new i_old
  i_new="$(grep -n '^## \[0.0.13\]' "$dir/CHANGELOG.md" | cut -d: -f1)"
  i_old="$(grep -n '^## \[0.0.1\]'  "$dir/CHANGELOG.md" | cut -d: -f1)"
  [[ "$i_new" -lt "$i_old" ]] || die "self-check: 新版本小节应排在历史版本之前"
  echo "self-check ok"
}

dry_run() { # ver
  local ver="$1" src="$ROOT" dst
  TMPCLEAN="$(mktemp -d)"
  dst="$TMPCLEAN"
  mkdir -p "$dst/server"
  cp "$src/package.json" "$dst/package.json"
  cp "$src/server/Cargo.toml" "$dst/server/Cargo.toml"
  cp "$src/CHANGELOG.md" "$dst/CHANGELOG.md"

  edit_package_json "$dst" "$ver"
  edit_cargo_toml "$dst" "$ver"
  edit_changelog "$dst" "$ver" "$(date -u +%F)"

  echo "==> 发布 v${ver} 将产生的文件变更："
  diff -u "$src/package.json"      "$dst/package.json"      || true
  diff -u "$src/server/Cargo.toml" "$dst/server/Cargo.toml" || true
  diff -u "$src/CHANGELOG.md"      "$dst/CHANGELOG.md"      || true
  echo "==> 演练结束（临时目录已清理，仓库未被改动）。"
  echo "    真正的发布还会执行：git commit chore(release): v${ver} → git tag v${ver} → git push（需 RELEASE_PAT）→ gh release create"
}

main() {
  local arg1="${1:-}" arg2="${2:-}" ver=""
  if [[ "$arg1" == "--self-check" ]]; then self_check; exit 0; fi
  if [[ "$arg1" == "--dry-run" ]]; then
    ver="$(resolve_version "$arg2")"
    ver="$(normalize_ver "$ver")"
    dry_run "$ver"
    exit 0
  fi

  # 完整发布：arg1 是版本号（留空则自动 patch 递增）
  ver="$(resolve_version "$arg1")"
  ver="$(normalize_ver "$ver")"
  cd "$ROOT"
  guard_branch
  validate_release "$ver"
  require_pat
  ci_gate
  edit_files "$ROOT" "$ver"
  git_ops "$ver"
  release_note "$ver"
  log "发布完成：v${ver}。docker.yml 将自动构建镜像，latest 随非预发布版本更新。"
}

main "$@"
