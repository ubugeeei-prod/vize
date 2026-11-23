#!/usr/bin/env bash
if [[ ! "$BASE_SHA" =~ ^0+$ ]]; then git fetch --no-tags origin "$BASE_SHA"; fi
node tools/support/compat/github/plan-tooling-tests.mjs "$BASE_SHA" "$GITHUB_SHA" --tier pr --output "$RUNNER_TEMP/tooling-plan.json"
native_build="$(node tools/support/compat/github/tooling-shard-native-build.ts "$RUNNER_TEMP/tooling-plan.json" "$TOOLING_SHARD")"
echo "native-build=$native_build" >> "$GITHUB_OUTPUT"
