#!/usr/bin/env bash
set -euo pipefail
CARGO_TARGET_DIR="$RUNNER_TEMP/instance-directive-build-head" cargo test --locked --profile ci-opt -p vize_canon --lib template_ts_directives -- --nocapture > "$RUNNER_TEMP/instance-directives/rust-tests.txt" 2>&1
node "$VIZE_CI_FILES/verify-rust-test-count.ts"
