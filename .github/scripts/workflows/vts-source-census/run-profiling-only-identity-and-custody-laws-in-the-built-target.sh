#!/usr/bin/env bash
set -euo pipefail
CARGO_TARGET_DIR="$RUNNER_TEMP/census-target" \
  cargo test --locked -p vize_canon --features profiling --lib profiling::tests
CARGO_TARGET_DIR="$RUNNER_TEMP/census-target" \
  cargo test --locked -p vize --features profiling --lib \
    commands::profile_export::tests::aliased_receipt_creation_fails_after_profile_write -- --exact
