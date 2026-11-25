#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/ci/workflows/03-ci-walltime/scripts/compile-identical-full-workspace-test-binaries-once.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
cargo test --locked --workspace --no-run
mkdir -p target/ci-walltime
cargo nextest list --locked --workspace --profile full --message-format json > target/ci-walltime/rust-selected-tests.json
