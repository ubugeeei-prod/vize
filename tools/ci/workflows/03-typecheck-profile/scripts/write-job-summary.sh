#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/ci/workflows/03-typecheck-profile/scripts/write-job-summary.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
if [[ -f "$RUNNER_TEMP/profile-out/summary.md" ]]; then
  head -c 900000 "$RUNNER_TEMP/profile-out/summary.md" >> "$GITHUB_STEP_SUMMARY"
fi
