#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/n8n-adoption/build-and-authenticate-the-current-source-lsp.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
cargo build --profile ci -p vize && vp node tests/differential/build-receipt.ts
