#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/lsp-dual-server-benchmark/build-and-receipt-the-exact-normal-vize-server.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
cargo build --profile ci -p vize
vp exec node tests/differential/build-receipt.ts
