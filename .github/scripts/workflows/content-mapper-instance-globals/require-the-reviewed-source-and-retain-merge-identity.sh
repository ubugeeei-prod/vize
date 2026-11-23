#!/usr/bin/env bash
test "$(git rev-parse HEAD)" = "$GITHUB_SHA"
if [ "$SOURCE_EVENT" = workflow_dispatch ]; then
  [[ "$EXPECTED_SOURCE_SHA" =~ ^[0-9a-f]{40}$ ]]
  test "$GITHUB_SHA" = "$EXPECTED_SOURCE_SHA"
else
  test "$SOURCE_EVENT" = pull_request
  test "$(git rev-parse HEAD^2)" = "$PR_HEAD"
fi
git diff --exit-code HEAD
mkdir -p "$RUNNER_TEMP/instance-globals-10617"
git rev-parse HEAD HEAD^{tree} > "$RUNNER_TEMP/instance-globals-10617/source-identity.txt"
