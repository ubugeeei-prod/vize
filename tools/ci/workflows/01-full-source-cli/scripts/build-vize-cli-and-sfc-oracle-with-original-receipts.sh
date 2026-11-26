#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/ci/workflows/01-full-source-cli/scripts/build-vize-cli-and-sfc-oracle-with-original-receipts.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
cargo build --profile ci -p vize && vp exec node tests/differential/build-receipt.ts && cat target/ci/vize.differential-build.json && cargo build --profile ci -p vize_atelier_sfc --example cli_build_result_oracle
