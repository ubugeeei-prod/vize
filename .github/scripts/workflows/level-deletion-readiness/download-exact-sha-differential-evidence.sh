#!/usr/bin/env bash
[[ "$EVIDENCE_RUN_ID" =~ ^[0-9]+$ ]]
gh api "repos/$GH_REPO/actions/runs/$EVIDENCE_RUN_ID" \
  --jq '{head_sha, conclusion, path, event}' > "$RUNNER_TEMP/evidence-run.json"
jq -e --arg sha "$GITHUB_SHA" \
  '.head_sha == $sha and .conclusion == "success" and .path == ".github/workflows/real-project-matrix.yml" and (.event == "schedule" or .event == "workflow_dispatch")' \
  "$RUNNER_TEMP/evidence-run.json"
gh run download "$EVIDENCE_RUN_ID" --name davinci-deletion-evidence \
  --dir "$RUNNER_TEMP/davinci-deletion-evidence"
