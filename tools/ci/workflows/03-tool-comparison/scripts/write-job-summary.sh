#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/ci/workflows/03-tool-comparison/scripts/write-job-summary.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
cat tool-benchmark-summary.md >> "$GITHUB_STEP_SUMMARY"
