#!/usr/bin/env bash
status=0
mkdir -p "$RUNNER_TEMP/jsx-authored-native"
for target in falsy_and nested_typecheck_runtime vue_jsx_vapor_compat_oracle dynamic_child_values; do
  cargo test --locked --profile ci -p vize_atelier_jsx --test "$target" -- --nocapture > "$RUNNER_TEMP/jsx-authored-native/$target.stdout.log" 2> "$RUNNER_TEMP/jsx-authored-native/$target.stderr.log" || status=1
done
VIZE_JSX_VAPOR_COMPAT_TRACE_CAPTURE="$RUNNER_TEMP/jsx-authored-native/reference-metadata.json" node tests/tooling/jsx-vapor-oracle/oracle.mjs --reference > "$RUNNER_TEMP/jsx-authored-native/reference.stdout.json" 2> "$RUNNER_TEMP/jsx-authored-native/reference.stderr.log" || status=1
exit "$status"
