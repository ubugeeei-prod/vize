#!/usr/bin/env bash
set -euo pipefail
CARGO_TARGET_DIR="$RUNNER_TEMP/partition-weight-target" cargo test --locked --profile ci-opt -p vize_canon --lib weight_work -- --nocapture --test-threads=1 > "$RUNNER_TEMP/partition-weight/work-laws.txt" 2>&1
cat "$RUNNER_TEMP/partition-weight/work-laws.txt"
