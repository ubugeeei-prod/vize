#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/lsp-dual-server-benchmark/collect-attribution-without-comparing-unlike-metrics.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
set -euo pipefail
mkdir -p "$LSP_DUAL_VIZE_PROFILE_DIR" "$LSP_DUAL_VUE_CPU_PROFILE_DIR"
vp exec node tests/performance/lsp-dual-server-benchmark.ts \
  --fresh 1 --warm 5 --out "$RUNNER_TEMP/lsp-attribution/report"
sha256sum "$VIZE_LSP_BIN" > "$RUNNER_TEMP/lsp-attribution/vize-binary.sha256"
git rev-parse HEAD > "$RUNNER_TEMP/lsp-attribution/source.sha"
vp exec node tests/performance/lsp-dual-server-attribution-manifest.ts
