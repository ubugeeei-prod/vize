#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/ci/workflows/03-lsp-resources/scripts/summarize.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
{
  echo "### TS-44 resource measurement (${RESOURCE_PRESET})"
  echo '```json'
  cat "${RUNNER_TEMP}/resource.json"
  echo '```'
} >> "$GITHUB_STEP_SUMMARY"
