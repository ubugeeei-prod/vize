#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/lsp-release-resources/summarize-the-separate-release-measurements.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
{
  echo '### Same-source shipped fat-LTO LSP supplement'
  echo 'The scheduled ci-opt gate retains its recipe and ceilings.'
  echo 'TS-44 uses three runs, 60 edits and 60s idle per run.'
  echo 'The separate 10k SFC workload is record-only, with all Corsa descendants included.'
  for result in release-build-identity release-resource release-lsp-resource; do
    echo "#### ${result}"
    echo '```json'
    cat "${RUNNER_TEMP}/${result}.json" || true
    echo '```'
  done
} >> "$GITHUB_STEP_SUMMARY"
