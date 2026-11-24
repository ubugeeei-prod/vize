#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/lsp-dual-server-benchmark/refuse-a-different-source-2.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
set -euo pipefail
test "$(git rev-parse HEAD)" = "$EXPECTED_SOURCE"
