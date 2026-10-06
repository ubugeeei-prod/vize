#!/usr/bin/env bash
set -euo pipefail
cd "$NATIVE_PHASE_SOURCE_ROOT"
export VIZE_EDITOR_CONFIG_CAPTURE="$RUNNER_TEMP/outside-import-types/editor-jsconfig"
mkdir -p "$VIZE_EDITOR_CONFIG_CAPTURE"
npm install --prefix "$RUNNER_TEMP/editor-jsconfig-provider" --ignore-scripts --no-package-lock --no-audit --no-fund vue@3.5.43
export VIZE_EDITOR_CONFIG_VUE_ROOT="$RUNNER_TEMP/editor-jsconfig-provider/node_modules/vue"
cargo test --locked --profile ci-opt -p vize_canon --lib jsconfig_tests:: -- --nocapture
VIZE_LSP_BIN="$NATIVE_PHASE_SOURCE_ROOT/target/ci/vize" VIZE_LSP_REQUIRE_SOURCE_BUILD=1 VIZE_TEST_REQUIRE_TSGO=1 VIZE_EDITOR_CONFIG_REQUIRED=1 vp node --test --test-concurrency=1 tests/tooling/lsp-editor-jsconfig.test.ts
