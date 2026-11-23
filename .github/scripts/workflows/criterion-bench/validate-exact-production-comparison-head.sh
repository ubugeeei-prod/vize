#!/usr/bin/env bash
set -euo pipefail
[[ "$BASE_SHA" =~ ^[0-9a-f]{40}$ ]]
[[ "$HEAD_SHA" =~ ^[0-9a-f]{40}$ ]]
[[ "$HEAD_SHA" == "$DISPATCH_SHA" ]]
[[ "$(git rev-parse HEAD)" == "$HEAD_SHA" ]]
git merge-base --is-ancestor "$BASE_SHA" "$HEAD_SHA"
