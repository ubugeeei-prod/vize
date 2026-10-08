#!/usr/bin/env bash
set -euo pipefail
unset VIZE_TEST_DISABLE_TSGO
export VIZE_TEST_REQUIRE_TSGO=1
export VIZE_INLAY_HINT_CAPTURE="$RUNNER_TEMP/outside-import-types/component-tag-types"
export VIZE_COMPONENT_TYPE_CAPTURE="$VIZE_INLAY_HINT_CAPTURE/type-query-custody"
cd "$NATIVE_PHASE_SOURCE_ROOT"
cargo test --locked --profile ci-opt -p vize --test lsp_component_tag_types_cli -- --nocapture
cargo test --locked --profile ci-opt -p vize --test lsp_tag_completion_cli -- --nocapture
cargo test --locked --profile ci-opt -p vize_maestro --lib tag_name_tests:: -- --nocapture
