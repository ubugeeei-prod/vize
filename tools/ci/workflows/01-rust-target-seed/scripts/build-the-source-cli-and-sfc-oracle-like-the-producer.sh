#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/ci/workflows/01-rust-target-seed/scripts/build-the-source-cli-and-sfc-oracle-like-the-producer.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
cargo build --profile ci -p vize && cargo build --profile ci -p vize_atelier_sfc --example cli_build_result_oracle
