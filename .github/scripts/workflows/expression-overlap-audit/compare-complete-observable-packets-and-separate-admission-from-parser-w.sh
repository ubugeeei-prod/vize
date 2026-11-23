#!/usr/bin/env bash
set -euo pipefail
export PROFILE_BINARY="$CARGO_TARGET_DIR/ci-opt/examples/failed_speculation_profile"
export GUARD_BINARY="$CARGO_TARGET_DIR/ci-opt/examples/fuzz_expression_10401_profile"
export HISTORICAL_OBSERVER="$CARGO_TARGET_DIR/ci-opt/examples/fuzz_expression_raw_close_observe"
sha256sum "$PROFILE_BINARY" "$GUARD_BINARY" > evidence/optimized-binaries.sha256
node "$VIZE_CI_FILES/collect-expression-packets.ts"
