#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/ci/workflows/03-instruction-counts/scripts/verify-native-descriptor-compatibility.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
cargo test --locked -p vize_l1 --lib container::vue
cargo test --locked -p vize_atelier_sfc --lib parse::tests
