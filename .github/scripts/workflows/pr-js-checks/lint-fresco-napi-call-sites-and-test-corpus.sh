#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/pr-js-checks/lint-fresco-napi-call-sites-and-test-corpus.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
cargo clippy --locked --profile ci -p vize_fresco --features napi --lib -- -D warnings
cargo clippy --locked --profile ci -p vize_fresco --lib --tests -- -D warnings
