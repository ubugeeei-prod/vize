#!/usr/bin/env bash
set -euo pipefail
export CARGO_TARGET_DIR="$RUNNER_TEMP/slot-head-target"
evidence="$RUNNER_TEMP/slot-evidence"
run_exact_test() {
  local label="$1"
  shift
  cargo test "$@" -- --exact --nocapture > "$evidence/$label.stdout" 2> "$evidence/$label.stderr"
  grep -Fxq "running 1 test" "$evidence/$label.stdout"
  grep -Fq "test result: ok. 1 passed; 0 failed; 0 ignored;" "$evidence/$label.stdout"
}
run_exact_test tuple-tests --locked --profile ci-opt -p vize_canon --lib virtual_ts::tests::slot_component_bindings::slot_constructor_calls_adapt_the_first_argument_without_replacing_the_signature
run_exact_test hoisted-owner --locked --profile ci-opt -p vize_canon --lib virtual_ts::tests::slot_component_bindings::hoisted_slot_constructor_branches_keep_the_component_owner
run_exact_test root-capture-scope --locked --profile ci-opt -p vize_canon --lib virtual_ts::expressions::constructor_instance::tests::constructor_capture_requires_props_and_a_root_slot_candidate
run_exact_test scoped-slots-snapshot --locked --profile ci-opt -p vize_canon --lib tests::virtual_ts_tests::snapshot_virtual_ts_scoped_slots
run_exact_test scoped-slot-expressions --locked --profile ci-opt -p vize_canon --lib virtual_ts::tests::test_scoped_slot_expressions
run_exact_test reserved-props-snapshot --locked --profile ci-opt -p vize_canon --lib virtual_ts::tests::test_reserved_prop_and_hyphenated_slot_names
run_exact_test constructor-packets --locked --profile ci-opt -p vize_canon --test constructor_emission_projection complete_constructor_projection_packets
