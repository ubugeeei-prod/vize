#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/typecheck-cold-pair/corpus-check-out-base-and-head-beside-the-driver.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
set -euo pipefail
git fetch --no-tags --depth=1 "https://github.com/$GITHUB_REPOSITORY.git" "$BASE_SHA" "$HEAD_SHA"
git worktree add --detach "$RUNNER_TEMP/pair-head" "$HEAD_SHA"
if [[ "$BASE_SHA" == "$HEAD_SHA" ]]; then
  ln -s "$RUNNER_TEMP/pair-head" "$RUNNER_TEMP/pair-base"
else
  git worktree add --detach "$RUNNER_TEMP/pair-base" "$BASE_SHA"
fi
