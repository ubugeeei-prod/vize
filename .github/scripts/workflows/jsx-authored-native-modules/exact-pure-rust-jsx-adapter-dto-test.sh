#!/usr/bin/env bash
set -euo pipefail
directory="$RUNNER_TEMP/jsx-authored-native"
test_name=napi::jsx::tests::jsx_compile_result_supports_ssr_output
cargo test --locked --profile ci -p vize_vitrine --lib "$test_name" -- --exact --list > "$directory/napi-dto.list.stdout" 2> "$directory/napi-dto.list.stderr"
node "$VIZE_CI_FILES/verify-jsx-dto-list.ts"
status=0
cargo test --locked --profile ci -p vize_vitrine --lib "$test_name" -- --exact > "$directory/napi-dto.run.stdout" 2> "$directory/napi-dto.run.stderr" || status=1
node "$VIZE_CI_FILES/record-jsx-dto-test.ts"
exit "$status"
