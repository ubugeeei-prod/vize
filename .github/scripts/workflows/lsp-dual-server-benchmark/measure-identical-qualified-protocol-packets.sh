#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/lsp-dual-server-benchmark/measure-identical-qualified-protocol-packets.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
set -euo pipefail
test -x "$CORSA_PATH"
test -x "$VIZE_LSP_BIN"
vp exec node tests/performance/lsp-dual-server-benchmark.ts \
  --fresh 100 --warm 100 --out "$RUNNER_TEMP/lsp-dual-server"
