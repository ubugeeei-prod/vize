#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/lsp-vs-volar/build-this-commit.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
cargo build --profile ci-opt -p vize
cp target/ci-opt/vize "$RUNNER_TEMP/vize-head"
