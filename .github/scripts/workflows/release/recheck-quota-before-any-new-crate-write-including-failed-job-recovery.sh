#!/usr/bin/env bash
git fetch --no-tags --depth=1 origin "$GITHUB_SHA"
mkdir -p "$RUNNER_TEMP/release-quota-tooling"
git archive "$GITHUB_SHA" package.json tools/support/release tools/support/archive tests/tooling/release/quota-live-legacy-refusal.ts | tar -x -C "$RUNNER_TEMP/release-quota-tooling"
node "$RUNNER_TEMP/release-quota-tooling/tools/support/release/release/main.ts" quota "$RELEASE_VERSION"
