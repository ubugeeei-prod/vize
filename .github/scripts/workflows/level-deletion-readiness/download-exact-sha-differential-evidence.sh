#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/level-deletion-readiness/download-exact-sha-differential-evidence.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
[[ "$EVIDENCE_RUN_ID" =~ ^[0-9]+$ ]]
gh api "repos/$GH_REPO/actions/runs/$EVIDENCE_RUN_ID" \
  --jq "$(<"$VIZE_CI_FILES/evidence-run-fields.jq")" > "$RUNNER_TEMP/evidence-run.json"
jq -e --arg sha "$GITHUB_SHA" \
  -f "$VIZE_CI_FILES/check-evidence-run.jq" \
  "$RUNNER_TEMP/evidence-run.json"
gh run download "$EVIDENCE_RUN_ID" --name davinci-deletion-evidence \
  --dir "$RUNNER_TEMP/davinci-deletion-evidence"
