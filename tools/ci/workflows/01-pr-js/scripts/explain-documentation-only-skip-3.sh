#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/ci/workflows/01-pr-js/scripts/explain-documentation-only-skip-3.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
echo 'Fresco and UI conformance skipped because this PR changes no package, compiler, or unknown source paths.' >> "$GITHUB_STEP_SUMMARY"
