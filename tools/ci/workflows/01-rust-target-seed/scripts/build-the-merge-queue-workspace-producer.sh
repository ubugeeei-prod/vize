#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/ci/workflows/01-rust-target-seed/scripts/build-the-merge-queue-workspace-producer.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
cargo clippy --workspace --profile ci -- -D warnings -D clippy::wildcard_imports
cargo test --workspace --profile ci --no-run
