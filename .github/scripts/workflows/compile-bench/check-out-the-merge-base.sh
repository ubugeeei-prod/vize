#!/usr/bin/env bash
set -euo pipefail
git fetch --no-tags --depth=50 origin main
base=$(git merge-base HEAD FETCH_HEAD)
# A fixed base measures a whole series against one starting point.
if [[ -n "${COMPILE_BENCH_BASE:-}" ]]; then
  git fetch --no-tags --depth=1 origin "$COMPILE_BENCH_BASE"
  base=$COMPILE_BENCH_BASE
fi
echo "BASE_SHA=$base" >> "$GITHUB_ENV"
git worktree add --detach "$RUNNER_TEMP/base" "$base"
