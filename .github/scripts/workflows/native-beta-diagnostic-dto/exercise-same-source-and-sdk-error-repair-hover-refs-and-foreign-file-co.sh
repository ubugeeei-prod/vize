#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/native-beta-diagnostic-dto/exercise-same-source-and-sdk-error-repair-hover-refs-and-foreign-file-co.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
set -euo pipefail
cd "$VIZE_NATIVE_DTO_SOURCE_ROOT"
test ! -e "$CARGO_TARGET_DIR"
cargo test --locked --profile ci -p vize_canon --lib lsp_client::editor_lsp::snapshot_source::tests::native_beta::actual_sdk_semantic_error_repair_navigation_and_two_file_scope_are_complete --no-run --message-format=json >"$VIZE_NATIVE_DTO_CAPTURE/build.jsonl" 2>"$VIZE_NATIVE_DTO_CAPTURE/build.stderr"
node "$GITHUB_WORKSPACE/tests/tooling/support/native-beta-dto/run-source.ts"
