#!/usr/bin/env bash
set -o pipefail
unset VIZE_TEST_DISABLE_TSGO
export CORSA_PATH="$RUNNER_TEMP/patina-tsrs/node_modules/@tsrs/linux-x64-gnu/tsrs"
cargo test --locked --profile ci -p vize --test integration lint_strict_boolean_cli:: -- --nocapture 2>&1 | tee target/patina-native-boolean/tsrs-tests.txt
