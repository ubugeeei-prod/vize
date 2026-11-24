#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/actions/test-native-canon/scripts/test-memberless-vue-preparation-against-its-acknowledged-same-owner-pull.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
set -euo pipefail
unset VIZE_TEST_DISABLE_TSGO
capture="$VIZE_SNAPSHOT_SOURCE_CAPTURE_DIR/memberless-preparation"
mkdir -p "$capture"
set +e
cargo test --locked --profile ci -p vize_canon --lib lsp_client::editor_lsp::configured_project::shared_transport::tests::native::replay::actual_memberless_vue_preparation_reattaches_same_editor_owner -- --ignored --exact --nocapture --color never >"$capture/test.stdout" 2>"$capture/test.stderr"
status=$?
set -e
printf '%s\n' "$status" >"$capture/test.status"
cat "$capture/test.stdout"
cat "$capture/test.stderr" >&2
node "$GITHUB_ACTION_PATH/scripts/test-memberless-vue-preparation-against-its-acknowledged-same-owner-pull.ts" "$capture"
