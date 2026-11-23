#!/usr/bin/env bash
set -euo pipefail
directory="$RUNNER_TEMP/typed-context-undefined"
set +e
cargo test --locked --profile ci -p vize_canon --lib virtual_ts::tests::context_undefined:: -- --nocapture --test-threads=1 > "$directory/ast.stdout" 2> "$directory/ast.stderr"
ast_status=$?
cargo test --locked --profile ci -p vize_canon --lib virtual_ts::tests::context_undefined::actual_native_context_options_keep_whole_clean_and_authored_error_packets -- --ignored --exact --nocapture --test-threads=1 > "$directory/options.stdout" 2> "$directory/options.stderr"
option_status=$?
cargo test --locked --profile ci -p vize --test integration content_mapper_tsgo_cli::typed_context_undefined::typed_context_values_preserve_complete_native_mapper_and_cli_packets -- --ignored --exact --nocapture --test-threads=1 > "$directory/after.stdout" 2> "$directory/after.stderr"
after_status=$?
set -e
printf '%s\n' "$ast_status" > "$directory/ast.status"
printf '%s\n' "$option_status" > "$directory/options.status"
printf '%s\n' "$after_status" > "$directory/after.status"
test "$ast_status" -eq 0
test "$option_status" -eq 0
test "$after_status" -eq 0
