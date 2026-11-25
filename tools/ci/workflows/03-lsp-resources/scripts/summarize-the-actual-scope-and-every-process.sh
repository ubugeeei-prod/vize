#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/ci/workflows/03-lsp-resources/scripts/summarize-the-actual-scope-and-every-process.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
{
  echo '### Production Maestro LSP: 10000 open SFCs, 16 metadata provider witnesses'
  echo 'Maestro and all Corsa descendants are included; the Node harness is excluded.'
  echo 'The isolated resident example and nine-file TS-44 baseline remain separate.'
  echo '```json'
  cat "${RUNNER_TEMP}/lsp-resource.json" || cat "${RUNNER_TEMP}/lsp-resource-smoke.json"
  echo '```'
} >> "$GITHUB_STEP_SUMMARY"
