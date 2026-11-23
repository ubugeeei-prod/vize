#!/usr/bin/env bash
set -euo pipefail
[[ "$REQUESTED_BASE" =~ ^[0-9a-f]{40}$ ]]
[[ "$REQUESTED_HEAD" =~ ^[0-9a-f]{40}$ ]]
[[ "$(git -C head rev-parse HEAD)" == "$REQUESTED_HEAD" ]]
if [[ "$GITHUB_EVENT_NAME" == "pull_request" ]]; then
  COMPARISON_BASE="$(git -C head merge-base "$REQUESTED_BASE" HEAD)"
else
  git -C head merge-base --is-ancestor "$REQUESTED_BASE" HEAD
  COMPARISON_BASE="$REQUESTED_BASE"
fi
git -C head worktree add ../base "$COMPARISON_BASE"
[[ "$(git -C base rev-parse HEAD)" == "$COMPARISON_BASE" ]]
mkdir -p measurements
export COMPARISON_BASE
node "$VIZE_CI_FILES/select-and-record-exact-comparison-commits.ts"
