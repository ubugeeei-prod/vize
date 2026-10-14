#!/usr/bin/env bash
# Required normal-factory authority; shared exact ci-opt producer and original gates.
set -euo pipefail
[[ "$GITHUB_EVENT_NAME" == pull_request ]] || exit 0
export VIZE_TEST_REQUIRE_TSGO=1
export VIZE_NATIVE_SYNTAX_CAPTURE_DIR="$RUNNER_TEMP/native-source-recipe-declaration-imports/packets"
unset VIZE_TEST_DISABLE_TSGO
cd "$NATIVE_PHASE_SOURCE_ROOT"
[[ "$(git rev-parse HEAD)" == "$SOURCE_SHA" ]]
capture="$RUNNER_TEMP/native-source-recipe-declaration-imports"
mkdir -p "$capture"
git rev-parse HEAD > "$capture/source.sha"
common=(--locked --profile ci-opt -p vize_canon --config 'profile.ci-opt.inherits="release"' --config 'profile.ci-opt.lto="thin"' --config 'profile.ci-opt.codegen-units=16')
for filter in declaration_sources:: included_declarations_stay_on_their_authored_path declaration_graph_identity::; do
  cargo test "${common[@]}" --lib "$filter" -- --list > "$capture/$filter.tests.txt"
  grep -E ': test$' "$capture/$filter.tests.txt"
  set +e
  cargo test "${common[@]}" --lib "$filter" -- --nocapture --test-threads=1 2>&1 | tee "$capture/$filter.log"
  statuses=("${PIPESTATUS[@]}")
  set -e
  printf '%s %s\n' "${statuses[0]}" "${statuses[1]}" > "$capture/$filter.status"
  (( statuses[0] == 0 && statuses[1] == 0 ))
done
declaration=vue_codegen::generic_component_tests::native::type_imports::
cargo test "${common[@]}" --lib "$declaration" -- --list > "$capture/declaration-tests.txt"
grep -Fx 'batch::virtual_project::vue_codegen::generic_component_tests::native::type_imports::declaration_barrel_and_direct_ts_vue_imports_keep_missing_neighbors_and_type_errors: test' "$capture/declaration-tests.txt"
set +e
cargo test "${common[@]}" --lib "$declaration" -- --nocapture --test-threads=1 2>&1 | tee "$capture/declaration.log"
statuses=("${PIPESTATUS[@]}")
set -e
printf '%s %s\n' "${statuses[0]}" "${statuses[1]}" > "$capture/declaration.status"
(( statuses[0] == 0 && statuses[1] == 0 ))
