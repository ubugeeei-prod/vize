#!/usr/bin/env bash
# Qualify actual custom argument expressions and unchanged built-in controls.
set -euo pipefail
[[ "$GITHUB_EVENT_NAME" == pull_request ]] || exit 0
unset VIZE_TEST_DISABLE_TSGO
export VIZE_TEST_REQUIRE_TSGO=1
export VIZE_CUSTOM_DIRECTIVE_CAPTURE="$RUNNER_TEMP/native-source-recipe-custom-directive-arguments"
mkdir -p "$VIZE_CUSTOM_DIRECTIVE_CAPTURE"
cd "$NATIVE_PHASE_SOURCE_ROOT"
cargo test --locked --profile ci-opt -p vize_canon \
  --test native_custom_directive_arguments --test native_spread_diagnostics \
  --config 'profile.ci-opt.inherits="release"' --config 'profile.ci-opt.lto="thin"' \
  --config 'profile.ci-opt.codegen-units=16' -- --nocapture --test-threads=1 \
  2>&1 | tee "$VIZE_CUSTOM_DIRECTIVE_CAPTURE/runtime.log"
