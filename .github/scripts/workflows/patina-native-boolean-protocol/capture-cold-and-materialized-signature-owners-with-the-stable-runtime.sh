#!/usr/bin/env bash
set -o pipefail
unset VIZE_TEST_DISABLE_TSGO
VIZE_SYMBOL_OWNER_CAPTURE="$GITHUB_WORKSPACE/target/patina-native-boolean/stable-owner.json" cargo test --locked --profile ci -p vize_patina --lib corsa_session::probe::signature_ -- --nocapture 2>&1 | tee target/patina-native-boolean/stable-owner-tests.txt
