#!/usr/bin/env bash
set -euo pipefail
[[ "$EXPECTED_SOURCE" =~ ^[0-9a-f]{40}$ ]]
test "$(git rev-parse HEAD)" = "$EXPECTED_SOURCE"
