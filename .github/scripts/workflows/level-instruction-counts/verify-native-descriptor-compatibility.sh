#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/level-instruction-counts/verify-native-descriptor-compatibility.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
cargo test --locked -p vize_l1 --lib container::vue
cargo test --locked -p vize_atelier_sfc --lib parse::tests
