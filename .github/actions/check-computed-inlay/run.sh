#!/usr/bin/env bash
set -euo pipefail
unset VIZE_TEST_DISABLE_TSGO
export VIZE_TEST_REQUIRE_TSGO=1
export VIZE_INLAY_HINT_CAPTURE="$RUNNER_TEMP/outside-import-types/computed-inlay"
cd "$NATIVE_PHASE_SOURCE_ROOT"
cargo test --locked --profile ci-opt -p vize --test lsp_computed_inlay_cli -- --nocapture
cargo test --locked --profile ci-opt -p vize_maestro --lib inlay_hint:: -- --nocapture
