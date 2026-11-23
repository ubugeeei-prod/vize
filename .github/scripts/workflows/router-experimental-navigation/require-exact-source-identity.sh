#!/usr/bin/env bash
test "$(git rev-parse HEAD)" = "$EXPECTED_SOURCE_SHA"
git diff --exit-code HEAD
mkdir -p "$RUNNER_TEMP/router-navigation"
git rev-parse HEAD HEAD^{tree} > "$RUNNER_TEMP/router-navigation/source-identity.txt"
