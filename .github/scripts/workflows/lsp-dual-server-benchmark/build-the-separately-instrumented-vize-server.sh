#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/lsp-dual-server-benchmark/build-the-separately-instrumented-vize-server.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
cargo build --profile ci-opt -p vize --features profiling
vp exec node tests/differential/build-receipt.ts --profile-ci-opt
