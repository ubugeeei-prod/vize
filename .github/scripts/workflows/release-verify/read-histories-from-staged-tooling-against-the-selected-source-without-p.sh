#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/release-verify/read-histories-from-staged-tooling-against-the-selected-source-without-p.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
[[ "$QUOTA_SOURCE" =~ ^[0-9a-f]{40}$ ]]
mkdir -p "$RUNNER_TEMP/release-quota-tooling"
git archive "$GITHUB_SHA" package.json tools/support/release tools/support/archive tests/tooling/release/quota-live-legacy-refusal.ts | tar -x -C "$RUNNER_TEMP/release-quota-tooling"
status=$(gh api "repos/$GITHUB_REPOSITORY/compare/$QUOTA_SOURCE...$GITHUB_SHA" --jq "$(<"$VIZE_CI_FILES/../../../../tools/ci/lib/scripts/commit-compare-status.jq")")
[[ "$status" == ahead || "$status" == identical ]]
git fetch --no-tags --depth=1 origin "$QUOTA_SOURCE"
git checkout --detach FETCH_HEAD
node "$RUNNER_TEMP/release-quota-tooling/tools/support/release/release/main.ts" quota-dry-run "$RELEASE_VERSION"
