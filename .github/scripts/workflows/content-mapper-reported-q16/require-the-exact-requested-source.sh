#!/usr/bin/env bash
if [ -n "$EXPECTED_SOURCE_SHA" ]; then
  test "$(git rev-parse HEAD)" = "$EXPECTED_SOURCE_SHA"
fi
mkdir -p "$RUNNER_TEMP/router-experimental"
git rev-parse HEAD HEAD^{tree} > "$RUNNER_TEMP/router-experimental/source-identity.txt"
git diff --exit-code HEAD
