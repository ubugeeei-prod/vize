#!/usr/bin/env bash
set -euo pipefail
[[ "$(git rev-parse HEAD)" == "$GITHUB_SHA" ]]
[[ "$(git -C .slot-provider-fixtures rev-parse HEAD)" == "b06a78673f41df01fe29d02bcb42b6b30073e451" ]]
git -C .slot-provider-fixtures diff --exit-code
mv .slot-provider-fixtures "$RUNNER_TEMP/slot-provider-fixtures"
