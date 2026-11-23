#!/usr/bin/env bash
set -euo pipefail
[[ "$BASE_SHA" =~ ^[0-9a-f]{40}$ ]]
[[ "$HEAD_SHA" =~ ^[0-9a-f]{40}$ ]]
[[ "$BASE_SHA" != "$HEAD_SHA" ]]
[[ "$HEAD_SHA" == "$DISPATCH_SHA" ]]
[[ "$(git -C base rev-parse HEAD)" == "$BASE_SHA" ]]
[[ "$(git -C head rev-parse HEAD)" == "$HEAD_SHA" ]]
git -C head merge-base --is-ancestor "$BASE_SHA" "$HEAD_SHA"
