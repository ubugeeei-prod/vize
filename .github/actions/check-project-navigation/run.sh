#!/usr/bin/env bash
set -euo pipefail
cd "$NATIVE_PHASE_SOURCE_ROOT"
export VIZE_EDITOR_CONFIG_CAPTURE="$RUNNER_TEMP/outside-import-types/project-navigation-8013"
export VIZE_PROJECT_NAVIGATION_VUE_ROOT="$RUNNER_TEMP/editor-jsconfig-provider/node_modules/vue"
cargo test --locked --profile ci-opt -p vize_maestro --lib workspace_project_files:: -- --nocapture
cargo test --locked --profile ci-opt -p vize_maestro --lib project::scope:: -- --nocapture
cargo test --locked --profile ci-opt -p vize_maestro --lib workspace_symbols::script::tests:: -- --nocapture
cargo test --locked --profile ci-opt -p vize_maestro --lib references::canonical::local_binding::tests:: -- --nocapture
set +e
VIZE_LSP_BIN="$NATIVE_PHASE_SOURCE_ROOT/target/ci/vize" VIZE_LSP_REQUIRE_SOURCE_BUILD=1 VIZE_TEST_REQUIRE_TSGO=1 VIZE_PROJECT_NAVIGATION_REQUIRED=1 vp node --test --test-concurrency=1 tests/tooling/lsp-project-navigation-8013.test.ts tests/tooling/lsp-workspace-semantic.test.ts tests/tooling/lsp-file-rename-workspace.test.ts
navigation_status=$?
set -e
if [[ -d "$NATIVE_PHASE_SOURCE_ROOT/target/differential/lsp-sessions" ]]; then
  cp -R "$NATIVE_PHASE_SOURCE_ROOT/target/differential/lsp-sessions" "$VIZE_EDITOR_CONFIG_CAPTURE/original-symbol-consumers"
fi
exit "$navigation_status"
