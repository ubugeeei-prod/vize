#!/usr/bin/env bash
set -euo pipefail
tool_args=()
if [[ -n "$SCALING_TOOLS" ]]; then tool_args+=(--tools "$SCALING_TOOLS"); fi
node tools/benchmarks/scripts/typecheck-scaling.ts \
  --target "$RUNNER_TEMP/scaling-target" --sha "$SHA" \
  --sizes "$SIZES" --runs "$RUNS" --giant-blocks "$GIANT_BLOCKS" \
  --monorepo-sfcs "$MONOREPO_SFCS" --nuxt-sizes "$NUXT_SIZES" \
  "${tool_args[@]}" \
  --run-url "$GITHUB_SERVER_URL/$GITHUB_REPOSITORY/actions/runs/$GITHUB_RUN_ID" \
  --work "$RUNNER_TEMP/scaling-work" --out "$RUNNER_TEMP/scaling-out"
