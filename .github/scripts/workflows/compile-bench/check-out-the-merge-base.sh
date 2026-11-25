#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/compile-bench/check-out-the-merge-base.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
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
