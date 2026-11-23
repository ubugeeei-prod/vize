#!/usr/bin/env bash
vp install --frozen-lockfile --prefer-offline
[[ "$BASE_SOURCE_SHA" =~ ^[0-9a-f]{40}$ ]]
git worktree add --detach target/finalization-compare-base "$BASE_SOURCE_SHA"
