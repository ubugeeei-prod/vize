#!/usr/bin/env bash
set -euo pipefail
unset VIZE_TEST_DISABLE_TSGO
qualifier_status=0
for control in anchors malformed-setup generic jsx native; do
  case "$control" in
    anchors) control_args=(--lib anchors::tests:: -- --nocapture) ;;
    malformed-setup) control_args=(--lib vue_codegen::malformed_setup_tests:: -- --nocapture --test-threads=1) ;;
    generic) control_args=(--lib vue_codegen::generic_component_tests:: -- --nocapture --test-threads=1) ;;
    jsx) control_args=(--lib jsx_codegen::nested_tests:: -- --nocapture) ;;
    native) control_args=(--test native_syntax_semantics -- --nocapture --test-threads=1) ;;
  esac
  set +e
  cargo test --locked --profile ci -p vize_canon "${control_args[@]}" 2>&1 | tee "target/native-syntax-completion/$control.log"
  control_status=("${PIPESTATUS[@]}")
  set -e
  printf '%s %s\n' "${control_status[0]}" "${control_status[1]}" > "target/native-syntax-completion/$control.status"
  if (( control_status[0] != 0 || control_status[1] != 0 )); then qualifier_status=1; fi
done
exit "$qualifier_status"
