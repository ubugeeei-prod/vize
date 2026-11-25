#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/tools/support/github/test-imported-raw-callback.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
set -euo pipefail

unset VIZE_TEST_DISABLE_TSGO
mkdir -p "$VIZE_IMPORTED_RAW_CAPTURE"
set +e
cargo test --locked --profile ci -p vize --test integration -- check_imported_raw_callback_cli::imported_mark_raw_getter_reads_keep_direct_callback_and_authored_error_controls --exact --nocapture --color never >"$VIZE_IMPORTED_RAW_CAPTURE/test.stdout" 2>"$VIZE_IMPORTED_RAW_CAPTURE/test.stderr"
status=$?
set -e
printf '%s\n' "$status" >"$VIZE_IMPORTED_RAW_CAPTURE/test.status"
cat "$VIZE_IMPORTED_RAW_CAPTURE/test.stdout"
cat "$VIZE_IMPORTED_RAW_CAPTURE/test.stderr" >&2
node tools/support/github/test-imported-raw-callback.ts check_imported_raw_callback_cli::imported_mark_raw_getter_reads_keep_direct_callback_and_authored_error_controls
