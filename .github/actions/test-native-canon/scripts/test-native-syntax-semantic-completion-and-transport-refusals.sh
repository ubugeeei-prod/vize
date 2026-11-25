#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/actions/test-native-canon/scripts/test-native-syntax-semantic-completion-and-transport-refusals.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
set -euo pipefail
unset VIZE_TEST_DISABLE_TSGO
cargo test --locked --profile ci -p vize_canon --test native_syntax_semantics -- --nocapture --test-threads=1
cargo test --locked --profile ci -p vize_canon --lib generated_diagnostics:: -- --nocapture --test-threads=1
