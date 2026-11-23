#!/usr/bin/env bash
set +e
cargo test --locked --profile ci -p vize_atelier_sfc --test native_scriptless_ssr -- --nocapture
rust_status=$?
vp node --test tests/tooling/native-sfc-scriptless-ssr-reference.test.ts
runtime_status=$?
set -e
test "$rust_status" -eq 0
test "$runtime_status" -eq 0
