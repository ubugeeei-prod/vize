#!/usr/bin/env bash
# Existing native phase build and recursive packet retention own this control.
set -euo pipefail
if [[ "$GITHUB_EVENT_NAME" != pull_request && "$GITHUB_EVENT_NAME" != schedule ]]; then exit 0; fi
unset VIZE_TEST_DISABLE_TSGO
export VIZE_TEST_REQUIRE_TSGO=1
export VIZE_NATIVE_SYNTAX_CAPTURE_DIR="$RUNNER_TEMP/native-source-recipe-transport-refusal"
cd "$NATIVE_PHASE_SOURCE_ROOT"
git submodule update --init --depth 1 -- tests/_fixtures/_git/element-plus
cargo test --locked --profile ci-opt -p vize_canon --test native_syntax_semantics \
  --config 'profile.ci-opt.inherits="release"' \
  --config 'profile.ci-opt.lto="thin"' \
  --config 'profile.ci-opt.codegen-units=16' -- --nocapture --test-threads=1
