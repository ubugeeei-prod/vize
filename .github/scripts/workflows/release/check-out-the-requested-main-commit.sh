#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/release/check-out-the-requested-main-commit.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
# The source names one commit by its full SHA, as the cadence and the runbook pass it.
if [[ ! "$RELEASE_SOURCE" =~ ^[0-9a-f]{40}$ ]]; then
  echo "source must be a full 40-character lowercase commit SHA" >&2
  exit 1
fi
# Only a commit already on main may be released.
status=$(gh api "repos/$GITHUB_REPOSITORY/compare/$RELEASE_SOURCE...$GITHUB_SHA" --jq "$(<"$VIZE_CI_FILES/../../../../tools/ci/lib/scripts/commit-compare-status.jq")")
[[ "$status" == ahead || "$status" == identical ]]
git fetch --depth=1 origin "$RELEASE_SOURCE"
git checkout --detach FETCH_HEAD
