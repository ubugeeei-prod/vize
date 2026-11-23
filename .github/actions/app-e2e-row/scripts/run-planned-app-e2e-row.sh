#!/usr/bin/env bash
set -euo pipefail
mkdir -p target/app-e2e-logs
timeout --signal=TERM --kill-after=15s "$PLANNED_TIMEOUT" \
  vp run --no-cache --filter './tests' "$PLANNED_TASK" 2>&1 \
  | tee "target/app-e2e-logs/$PLAN_PROFILE-$PLAN_SUITE-$PLAN_SHARD.log"
