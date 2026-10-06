#!/usr/bin/env bash
set -euo pipefail
cd "$NATIVE_PHASE_SOURCE_ROOT"
export VIZE_PRIVATE_IMPORT_CAPTURE="$RUNNER_TEMP/outside-import-types/package-private-imports"
mkdir -p "$VIZE_PRIVATE_IMPORT_CAPTURE"
npm install --prefix "$RUNNER_TEMP/package-private-import-provider" --ignore-scripts --no-package-lock --no-audit --no-fund vue@3.5.43
export VIZE_PRIVATE_IMPORT_VUE_ROOT="$RUNNER_TEMP/package-private-import-provider/node_modules/vue"
cargo test --locked --profile ci-opt -p vize_canon --lib private_import_tests:: -- --nocapture
VIZE_LSP_BIN="$NATIVE_PHASE_SOURCE_ROOT/target/ci/vize" VIZE_LSP_REQUIRE_SOURCE_BUILD=1 VIZE_TEST_REQUIRE_TSGO=1 VIZE_PRIVATE_IMPORT_REQUIRED=1 vp node --test --test-concurrency=1 tests/tooling/lsp-package-private-imports.test.ts tests/tooling/lsp-package-private-imports-js.test.ts
