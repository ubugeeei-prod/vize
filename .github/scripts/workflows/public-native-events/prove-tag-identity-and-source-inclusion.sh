#!/usr/bin/env bash
[[ "$PUBLIC_VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]
[[ "$PUBLIC_SOURCE_SHA" =~ ^[0-9a-f]{40}$ ]]
[[ "$PUBLIC_PHASE" == before || "$PUBLIC_PHASE" == after ]]
mkdir -p "$RUNNER_TEMP/event-receipts"
git fetch --no-tags --depth=1 origin "refs/tags/v$PUBLIC_VERSION"
[[ "$(git rev-parse 'FETCH_HEAD^{commit}')" == "$PUBLIC_SOURCE_SHA" ]]
gh api "repos/$GITHUB_REPOSITORY/compare/$REQUIRED_SOURCE...$PUBLIC_SOURCE_SHA" > "$RUNNER_TEMP/event-receipts/source-comparison.json"
node "$VIZE_CI_FILES/prove-tag-identity-and-source-inclusion.ts"
