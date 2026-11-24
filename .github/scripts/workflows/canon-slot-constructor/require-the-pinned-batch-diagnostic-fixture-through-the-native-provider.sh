#!/usr/bin/env bash
set -euo pipefail
[[ -z "${VIZE_TEST_DISABLE_TSGO+x}" ]]
mkdir -p tests/node_modules
ln -s "$RUNNER_TEMP/slot-provider/node_modules/vue" tests/node_modules/vue
checker=$(node "$VIZE_CI_FILES/require-the-pinned-batch-diagnostic-fixture-through-the-native-provider.ts" "$RUNNER_TEMP/slot-provider/node_modules/typescript-native/lib/getExePath.js")
[[ -f "$checker" ]]
export CARGO_TARGET_DIR="$RUNNER_TEMP/slot-head-target"
VIZE_TEST_REQUIRE_TSGO=1 CORSA_PATH="$checker" cargo test --locked --profile ci-opt -p vize_canon --test zero_argument_slot_constructor_diagnostics zero_argument_slot_constructors_preserve_mapper_and_authored_diagnostics -- --exact --nocapture > "$RUNNER_TEMP/slot-evidence/fix-history.stdout" 2> "$RUNNER_TEMP/slot-evidence/fix-history.stderr"
VIZE_TEST_REQUIRE_TSGO=1 CORSA_PATH="$checker" cargo test --locked --profile ci-opt -p vize_canon --lib batch::type_checker::tests::recent_issues::generic_constructor_slots::correlated_props::constructor_props_follow_each_usage_in_both_overload_orders -- --exact --nocapture > "$RUNNER_TEMP/slot-evidence/correlated-props.stdout" 2> "$RUNNER_TEMP/slot-evidence/correlated-props.stderr"
grep -Fxq 'running 1 test' "$RUNNER_TEMP/slot-evidence/correlated-props.stdout"
grep -Fq 'test result: ok. 1 passed; 0 failed; 0 ignored;' "$RUNNER_TEMP/slot-evidence/correlated-props.stdout"
node "$VIZE_CI_FILES/verify-native-batch.ts"
