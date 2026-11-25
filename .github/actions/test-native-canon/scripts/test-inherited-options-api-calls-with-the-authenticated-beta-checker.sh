#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/actions/test-native-canon/scripts/test-inherited-options-api-calls-with-the-authenticated-beta-checker.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
set -euo pipefail
unset VIZE_TEST_DISABLE_TSGO
capture="$VIZE_SNAPSHOT_SOURCE_CAPTURE_DIR"
mkdir -p "$capture"
node "$GITHUB_ACTION_PATH/scripts/test-inherited-options-api-calls-with-the-authenticated-beta-checker.ts" "$CORSA_PATH" "$capture"
set +e
cargo test --locked --profile ci -p vize_canon --features cache-custody --lib batch::type_checker::tests::recent_issues::options_api_inherited_template_members:: -- --nocapture
api_status=$?
cargo test --locked --profile ci -p vize --test integration -- check_options_api_inherited_members_cli:: --nocapture
cli_status=$?
set -e
printf '%s\n' "$api_status" >"$capture/api.status"
printf '%s\n' "$cli_status" >"$capture/cli.status"
sha256sum "$CORSA_PATH" >"$capture/native.after.sha256"
test "$(cut -d ' ' -f 1 "$capture/native.after.sha256")" = a183b1956edbe7e7d98f2249858eedf2d8a9790809cc4909db1276e202faaa70
test "$api_status" -eq 0
test "$cli_status" -eq 0
