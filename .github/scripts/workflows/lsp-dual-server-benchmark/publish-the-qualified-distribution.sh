#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/lsp-dual-server-benchmark/publish-the-qualified-distribution.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
if [[ -f "$RUNNER_TEMP/lsp-dual-server/summary.md" ]]; then
  cat "$RUNNER_TEMP/lsp-dual-server/summary.md" >> "$GITHUB_STEP_SUMMARY"
fi
