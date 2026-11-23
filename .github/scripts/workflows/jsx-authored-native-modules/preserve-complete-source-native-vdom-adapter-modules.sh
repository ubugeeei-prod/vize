#!/usr/bin/env bash
status=0
for name in jsx_compile_result_includes_runtime_helper_preamble jsx_compile_result_wraps_block_body_setup_state; do
  cargo test --locked --profile ci -p vize_vitrine --lib "napi::jsx::tests::$name" -- --exact > "$RUNNER_TEMP/jsx-authored-native/napi-vdom-$name.stdout" 2> "$RUNNER_TEMP/jsx-authored-native/napi-vdom-$name.stderr" || status=1
done
exit "$status"
