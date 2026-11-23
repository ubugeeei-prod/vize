#!/usr/bin/env bash
if [[ -f "$RUNNER_TEMP/lsp-dual-server/summary.md" ]]; then
  cat "$RUNNER_TEMP/lsp-dual-server/summary.md" >> "$GITHUB_STEP_SUMMARY"
fi
