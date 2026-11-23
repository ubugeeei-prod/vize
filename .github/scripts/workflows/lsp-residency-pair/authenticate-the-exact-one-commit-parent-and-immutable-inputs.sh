#!/usr/bin/env bash
set -euo pipefail
[[ "$BASE_SHA" =~ ^[0-9a-f]{40}$ ]]
[[ "$SOURCE_SHA" =~ ^[0-9a-f]{40}$ ]]
test "$(git rev-parse HEAD)" = "$SOURCE_SHA"
test "$(git rev-parse "$SOURCE_SHA^")" = "$BASE_SHA"
git diff --exit-code "$BASE_SHA" "$SOURCE_SHA" -- Cargo.lock pnpm-lock.yaml package.json
