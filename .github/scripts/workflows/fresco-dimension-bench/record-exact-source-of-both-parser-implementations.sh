#!/usr/bin/env bash
set -euo pipefail
[[ "$HEAD_SHA" =~ ^[0-9a-f]{40}$ ]]
[[ "$(git rev-parse HEAD)" == "$HEAD_SHA" ]]
echo "BASE_SHA=$HEAD_SHA" >> "$GITHUB_ENV"
