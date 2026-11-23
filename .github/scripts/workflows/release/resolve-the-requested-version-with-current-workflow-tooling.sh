#!/usr/bin/env bash
mkdir -p "$RUNNER_TEMP/release-quota-tooling"
git archive "$GITHUB_SHA" package.json tools/support/release tools/support/archive tests/tooling/release/quota-live-legacy-refusal.ts | tar -x -C "$RUNNER_TEMP/release-quota-tooling"
quota_main="$RUNNER_TEMP/release-quota-tooling/tools/support/release/release/main.ts"
version="$RELEASE_VERSION"
if [[ -z "$version" ]]; then
  version="$(node "$quota_main" next)"
fi
echo "version=$version" >> "$GITHUB_OUTPUT"
