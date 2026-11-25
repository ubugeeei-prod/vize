#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/davinci-resource-budgets/summarize.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
{
  echo "### TS-44 resource measurement (${RESOURCE_PRESET})"
  echo '```json'
  cat "${RUNNER_TEMP}/resource.json"
  echo '```'
} >> "$GITHUB_STEP_SUMMARY"
