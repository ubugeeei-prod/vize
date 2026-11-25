#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/ci/workflows/03-typecheck-scaling/scripts/check-out-the-measured-commit-beside-the-driver.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
set -euo pipefail
git fetch --no-tags --depth=1 "https://github.com/$GITHUB_REPOSITORY.git" "$SHA"
git worktree add --detach "$RUNNER_TEMP/scaling-target" "$SHA"
