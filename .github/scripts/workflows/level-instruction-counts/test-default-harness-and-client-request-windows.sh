#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/level-instruction-counts/test-default-harness-and-client-request-windows.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
cargo test --locked -p davinci_harness --lib
cargo test --locked -p davinci_harness --lib --features instruction-counts
