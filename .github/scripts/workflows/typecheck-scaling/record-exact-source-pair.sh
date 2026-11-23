#!/usr/bin/env bash
set -euo pipefail
[[ "$(git -C base rev-parse HEAD)" == "$BASE_SHA" ]]
[[ "$(git -C head rev-parse HEAD)" == "$HEAD_SHA" ]]
[[ "$(git -C driver rev-parse HEAD)" == "$DRIVER_SHA" ]]
mkdir -p paired-scaling/build
