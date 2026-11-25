#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/ci/workflows/07-proofs/scripts/produce-literal-bytes-from-the-compiler-writer.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
cargo test --locked --profile ci-opt -p vize_l0 --lib js_string::tests
cargo test --locked --profile ci-opt -p vize_l0 --test emitted_string_literals
