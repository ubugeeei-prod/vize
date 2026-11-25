#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/ci/workflows/03-lsp-resources/scripts/summarize-the-isolated-session.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
{
  echo '### Synthetic resident database (fixture alpha inputs; no LSP or Corsa)'
  echo '```json'
  cat "${RUNNER_TEMP}/resident-resource.json"
  echo '```'
} >> "$GITHUB_STEP_SUMMARY"
