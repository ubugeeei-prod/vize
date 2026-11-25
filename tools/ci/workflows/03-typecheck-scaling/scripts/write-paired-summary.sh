#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/ci/workflows/03-typecheck-scaling/scripts/write-paired-summary.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
if [[ -f paired-scaling/measurements/summary.md ]]; then
  cat paired-scaling/measurements/summary.md >> "$GITHUB_STEP_SUMMARY"
fi
