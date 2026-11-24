#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/actions/test-native-lsp/scripts/build-the-same-source-cli-and-record-its-identity.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
cargo build --locked --profile ci -p vize
node tests/differential/build-receipt.ts
