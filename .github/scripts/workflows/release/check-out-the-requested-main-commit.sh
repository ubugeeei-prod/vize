#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/release/check-out-the-requested-main-commit.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
# Only a commit already on main may be released.
status=$(gh api "repos/$GITHUB_REPOSITORY/compare/$RELEASE_SOURCE...$GITHUB_SHA" --jq "$(<"$VIZE_CI_FILES/../shared/commit-compare-status.jq")")
[[ "$status" == ahead || "$status" == identical ]]
git fetch --depth=1 origin "$RELEASE_SOURCE"
git checkout --detach FETCH_HEAD
