#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/typecheck-profile/profile-the-lanes.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
set -euo pipefail
args=(--head "$RUNNER_TEMP/profile-head" --head-sha "$HEAD_SHA")
if [[ -n "$BASE_SHA" ]]; then args+=(--base "$RUNNER_TEMP/profile-base" --base-sha "$BASE_SHA"); fi
node tools/benchmarks/scripts/typecheck-profile.ts "${args[@]}" \
  --corpora "$CORPORA" --lanes "$LANES" --frequency "$FREQUENCY" \
  --perf "$PERF_BIN" --flamegraph "$RUNNER_TEMP/flamegraph" \
  --work "$RUNNER_TEMP/profile-work" --out "$RUNNER_TEMP/profile-out"
