#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/actions/app-e2e-row/scripts/run-planned-app-e2e-row.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
set -euo pipefail
mkdir -p target/app-e2e-logs
timeout --signal=TERM --kill-after=15s "$PLANNED_TIMEOUT" \
  vp run --no-cache --filter './tests' "$PLANNED_TASK" 2>&1 \
  | tee "target/app-e2e-logs/$PLAN_PROFILE-$PLAN_SUITE-$PLAN_SHARD.log"
