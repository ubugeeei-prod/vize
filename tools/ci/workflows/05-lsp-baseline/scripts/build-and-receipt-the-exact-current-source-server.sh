#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/ci/workflows/05-lsp-baseline/scripts/build-and-receipt-the-exact-current-source-server.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
cargo build --profile ci -p vize
node tests/differential/build-receipt.ts
