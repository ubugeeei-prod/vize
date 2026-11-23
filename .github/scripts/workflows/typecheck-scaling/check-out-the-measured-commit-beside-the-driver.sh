#!/usr/bin/env bash
set -euo pipefail
git fetch --no-tags --depth=1 "https://github.com/$GITHUB_REPOSITORY.git" "$SHA"
git worktree add --detach "$RUNNER_TEMP/scaling-target" "$SHA"
