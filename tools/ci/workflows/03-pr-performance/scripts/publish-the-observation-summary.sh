#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/ci/workflows/03-pr-performance/scripts/publish-the-observation-summary.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
if [[ -f performance-observation/measurements/summary.md ]]; then
  cat performance-observation/measurements/summary.md >> "$GITHUB_STEP_SUMMARY"
else
  printf '%s\n' '### PR performance observation' '' 'No qualified comparison was produced. Inspect the failed measurement/build step and retained evidence; this is not a performance improvement.' >> "$GITHUB_STEP_SUMMARY"
fi
