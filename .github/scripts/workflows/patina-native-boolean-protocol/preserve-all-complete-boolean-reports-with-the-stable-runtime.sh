#!/usr/bin/env bash
set -o pipefail
unset VIZE_TEST_DISABLE_TSGO
cargo test --locked --profile ci -p vize --test integration lint_strict_boolean_cli:: -- --nocapture 2>&1 | tee target/patina-native-boolean/stable-tests.txt
