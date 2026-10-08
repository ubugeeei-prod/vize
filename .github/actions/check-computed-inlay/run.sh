#!/usr/bin/env bash
set -euo pipefail
unset VIZE_TEST_DISABLE_TSGO
export VIZE_TEST_REQUIRE_TSGO=1
export VIZE_INLAY_HINT_CAPTURE="$RUNNER_TEMP/outside-import-types/computed-inlay"
cd "$NATIVE_PHASE_SOURCE_ROOT"
cargo test --locked --profile ci-opt -p vize --test lsp_computed_inlay_cli -- --nocapture
cargo test --locked --profile ci-opt -p vize_maestro --lib inlay_hint:: -- --nocapture
cargo test --locked --profile ci-opt -p vize_croquis --lib builtin_types:: -- --nocapture
cargo test --locked --profile ci-opt -p vize_croquis --lib builtin_types_default_analysis_keeps_whole_output_without_fact_storage -- --nocapture
cargo test --locked --profile ci-opt -p vize_croquis --lib fused_resolution_ -- --nocapture
cargo build --profile ci -p vize
vp node tests/differential/build-receipt.ts
scorecard_status=0
VIZE_LSP_BIN="$NATIVE_PHASE_SOURCE_ROOT/target/ci/vize" VIZE_LSP_REQUIRE_SOURCE_BUILD=1 \
  vp node --test --test-concurrency=1 tests/tooling/lsp-vue-language-tools-oracles.test.ts || scorecard_status=$?
mkdir -p "$VIZE_INLAY_HINT_CAPTURE/editor-only-scorecard"
cp -R target/differential/lsp-sessions "$VIZE_INLAY_HINT_CAPTURE/editor-only-scorecard/"
cp target/ci/vize.differential-build.json "$VIZE_INLAY_HINT_CAPTURE/editor-only-scorecard/build-receipt.json"
feature_isolation_status=0
VIZE_LSP_BIN="$NATIVE_PHASE_SOURCE_ROOT/target/ci/vize" VIZE_LSP_REQUIRE_SOURCE_BUILD=1 vp node --test --test-concurrency=1 tests/tooling/lsp-editor-feature-isolation.test.ts || feature_isolation_status=$?
mkdir -p "$VIZE_INLAY_HINT_CAPTURE/editor-feature-isolation"
cp -R target/differential/lsp-sessions "$VIZE_INLAY_HINT_CAPTURE/editor-feature-isolation/"
cp target/ci/vize.differential-build.json "$VIZE_INLAY_HINT_CAPTURE/editor-feature-isolation/build-receipt.json"
if [[ "$scorecard_status" != 0 ]]; then exit "$scorecard_status"; fi
exit "$feature_isolation_status"
