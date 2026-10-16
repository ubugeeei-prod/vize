#!/usr/bin/env bash
# Reuse the existing Canon ci-opt profile, native runtime and sticky target.
set -euo pipefail
if [[ "$GITHUB_EVENT_NAME" != pull_request && "$GITHUB_EVENT_NAME" != schedule ]]; then exit 0; fi
unset VIZE_TEST_DISABLE_TSGO
export VIZE_TEST_REQUIRE_TSGO=1
export VIZE_NATIVE_SYNTAX_CAPTURE_DIR="$RUNNER_TEMP/native-source-recipe-empty-vue-star"
mkdir -p "$VIZE_NATIVE_SYNTAX_CAPTURE_DIR"
cd "$NATIVE_PHASE_SOURCE_ROOT"
options=(--locked --profile ci-opt -p vize_canon --test empty_vue_star_exports)
options+=(--config 'profile.ci-opt.inherits="release"' --config 'profile.ci-opt.lto="thin"')
options+=(--config 'profile.ci-opt.codegen-units=16')
status=0
cargo test "${options[@]}" --no-run --message-format=json \
  > "$VIZE_NATIVE_SYNTAX_CAPTURE_DIR/builder.ndjson" \
  2> "$VIZE_NATIVE_SYNTAX_CAPTURE_DIR/builder.stderr" || status=$?
cat "$VIZE_NATIVE_SYNTAX_CAPTURE_DIR/builder.stderr" >&2
if [[ "$status" != 0 ]]; then cat "$VIZE_NATIVE_SYNTAX_CAPTURE_DIR/builder.ndjson" >&2; exit "$status"; fi
node tools/support/compat/github/native-empty-vue-star-exports.mjs
