#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/ci/workflows/03-typecheck-scaling/scripts/write-job-summary.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
if [[ -f "$RUNNER_TEMP/scaling-out/summary.md" ]]; then cat "$RUNNER_TEMP/scaling-out/summary.md" >> "$GITHUB_STEP_SUMMARY"; fi
