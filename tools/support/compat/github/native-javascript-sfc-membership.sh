#!/usr/bin/env bash
# Execute the admitted project-membership and ownership laws on the original native recipe.
set -euo pipefail
if [[ "$GITHUB_EVENT_NAME" != pull_request && "$GITHUB_EVENT_NAME" != schedule ]]; then exit 0; fi
unset VIZE_TEST_DISABLE_TSGO
export VIZE_TEST_REQUIRE_TSGO=1
export VIZE_UNKNOWN_COMPAT_CAPTURE="$RUNNER_TEMP/native-source-recipe-javascript-sfc"
mkdir -p "$VIZE_UNKNOWN_COMPAT_CAPTURE"
cd "$NATIVE_PHASE_SOURCE_ROOT"
for specification in recent_issues::javascript_sfc_membership:2 artifacts::javascript_collision_tests:2 batch::source_policy::tests:2 artifacts::options_projection_tests:2 recent_issues::options_projection:1 options_api_required_props::reports_cross_file_mismatch_for_runtime_constructor_props:1; do
  filter="${specification%:*}"
  expected="${specification##*:}"
  packet="$VIZE_UNKNOWN_COMPAT_CAPTURE/${filter//:/_}.log"
  cargo test --locked --profile ci-opt -p vize_canon --lib "$filter" \
    --config 'profile.ci-opt.inherits="release"' \
    --config 'profile.ci-opt.lto="thin"' \
    --config 'profile.ci-opt.codegen-units=16' -- --nocapture --test-threads=1 2>&1 | tee "$packet"
  grep -F "test result: ok. $expected passed; 0 failed; 0 ignored;" "$packet"
done
