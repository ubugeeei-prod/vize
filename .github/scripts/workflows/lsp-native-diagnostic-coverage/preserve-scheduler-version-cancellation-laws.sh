#!/usr/bin/env bash
set -euo pipefail
capture="$GITHUB_WORKSPACE/target/vize-tests/diagnostic-capacity-129"
cargo test --locked --profile ci -p vize_maestro --lib server::initial_diagnostics:: 2>&1 | tee "$capture/scheduler-current.log"
node "$VIZE_CI_FILES/verify-scheduler-laws.ts" "$capture/scheduler-current.log"
