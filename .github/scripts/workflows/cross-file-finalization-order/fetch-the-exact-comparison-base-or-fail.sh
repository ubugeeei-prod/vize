#!/usr/bin/env bash
set -euo pipefail
[[ "$BASE_SOURCE_SHA" =~ ^[0-9a-f]{40}$ ]]
git fetch --no-tags --depth=1 origin "$BASE_SOURCE_SHA"
