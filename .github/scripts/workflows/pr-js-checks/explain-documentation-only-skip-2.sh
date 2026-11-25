#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/pr-js-checks/explain-documentation-only-skip-2.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
echo 'Oxlint project qualification skipped because this PR changes no package, compiler, or unknown source paths.' >> "$GITHUB_STEP_SUMMARY"
