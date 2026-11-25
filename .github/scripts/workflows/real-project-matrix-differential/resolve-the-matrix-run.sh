#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/real-project-matrix-differential/resolve-the-matrix-run.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
[[ "$RUN_ID" =~ ^[0-9]+$ ]]
gh api "repos/$GITHUB_REPOSITORY/actions/runs/$RUN_ID" > "$RUNNER_TEMP/matrix-run.json"
jq -e -f "$VIZE_CI_FILES/check-matrix-run.jq" "$RUNNER_TEMP/matrix-run.json"
jq -r -f "$VIZE_CI_FILES/matrix-outputs.jq" "$RUNNER_TEMP/matrix-run.json" >> "$GITHUB_OUTPUT"
