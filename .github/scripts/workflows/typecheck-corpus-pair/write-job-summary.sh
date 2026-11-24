#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/typecheck-corpus-pair/write-job-summary.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
for report in cold summary projection reply-shape attribution batch-vs-mapper; do
  if [[ -f "$RUNNER_TEMP/pair-out/$report.md" ]]; then cat "$RUNNER_TEMP/pair-out/$report.md" >> "$GITHUB_STEP_SUMMARY"; fi
done
