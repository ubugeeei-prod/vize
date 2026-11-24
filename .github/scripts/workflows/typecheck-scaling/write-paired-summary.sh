#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/typecheck-scaling/write-paired-summary.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
if [[ -f paired-scaling/measurements/summary.md ]]; then
  cat paired-scaling/measurements/summary.md >> "$GITHUB_STEP_SUMMARY"
fi
