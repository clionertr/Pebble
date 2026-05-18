#!/usr/bin/env bash
# Build Pebble Docker images with cache busting.
# Usage:
#   ./deploy/build.sh              # Build with default cache
#   ./deploy/build.sh --no-cache   # Force fresh rebuild

set -euo pipefail

cd "$(dirname "$0")/.."

if [[ "${1:-}" == "--no-cache" ]]; then
  echo "==> Building with --no-cache (full rebuild)..."
  CACHEBUST="$(date +%s)" docker compose build --no-cache
else
  echo "==> Building with incremental cache..."
  echo "    (COPY . . 自动检测源码变更；传 CACHEBUST=\$(date +%s) 可强制重编译)"
  CACHEBUST="${CACHEBUST:-0}" docker compose build
fi

echo "==> Build complete."
echo "    Run: docker compose up -d --force-recreate"
echo "    or:  docker compose up -d"
