#!/usr/bin/env bash
set -euo pipefail
git fetch --no-tags --depth=1 "https://github.com/$GITHUB_REPOSITORY.git" $HEAD_SHA $BASE_SHA
git worktree add --detach "$RUNNER_TEMP/profile-head" "$HEAD_SHA"
if [[ -n "$BASE_SHA" ]]; then git worktree add --detach "$RUNNER_TEMP/profile-base" "$BASE_SHA"; fi
