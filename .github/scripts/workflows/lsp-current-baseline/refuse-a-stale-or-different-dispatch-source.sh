#!/usr/bin/env bash
set -euo pipefail
[[ "$VIZE_BASELINE_EXPECTED_SOURCE" =~ ^[0-9a-f]{40}$ ]]
test "$(git rev-parse HEAD)" = "$VIZE_BASELINE_EXPECTED_SOURCE"
test "$GITHUB_SHA" = "$VIZE_BASELINE_EXPECTED_SOURCE"
