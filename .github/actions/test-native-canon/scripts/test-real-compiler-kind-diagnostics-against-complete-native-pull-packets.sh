#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/actions/test-native-canon/scripts/test-real-compiler-kind-diagnostics-against-complete-native-pull-packets.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
set -euo pipefail
unset VIZE_TEST_DISABLE_TSGO
capture="$VIZE_SNAPSHOT_SOURCE_CAPTURE_DIR/compiler-kinds"
mkdir -p "$capture"
set +e
cargo test --locked --profile ci -p vize_canon --lib lsp_client::editor_lsp::snapshot_source::tests::native::actual_native_compiler_kind_reads_match_whole_ts_and_generated_vue_pull_packets -- --ignored --exact --nocapture --color never >"$capture/test.stdout" 2>"$capture/test.stderr"
status=$?
set -e
printf '%s\n' "$status" >"$capture/test.status"
cat "$capture/test.stdout"
cat "$capture/test.stderr" >&2
node "$GITHUB_ACTION_PATH/scripts/test-real-compiler-kind-diagnostics-against-complete-native-pull-packets.ts" "$capture"
