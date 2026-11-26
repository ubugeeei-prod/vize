#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/ci/workflows/01-pr-source/scripts/explain-unrelated-skip-3.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
echo 'Playground browser tests skipped because this PR changes no compiler, Vite, or playground source paths.' >> "$GITHUB_STEP_SUMMARY"
