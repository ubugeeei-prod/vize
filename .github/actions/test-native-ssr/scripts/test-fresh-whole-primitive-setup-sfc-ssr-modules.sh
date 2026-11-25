#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/actions/test-native-ssr/scripts/test-fresh-whole-primitive-setup-sfc-ssr-modules.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
set +e
cargo test --locked --profile ci -p vize_atelier_sfc --test native_scriptless_setup_ssr -- --nocapture
rust_status=$?
vp node --test tests/tooling/native-sfc-setup-ssr-reference.test.ts tests/tooling/native-sfc-setup-ssr-direct-child-reference.test.ts
runtime_status=$?
set -e
test "$rust_status" -eq 0
test "$runtime_status" -eq 0
