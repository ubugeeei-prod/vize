#!/usr/bin/env bash
set -euo pipefail
[[ "$BENCHMARK_BASE_SHA" =~ ^[0-9a-f]{40}$ && "$BENCHMARK_HEAD_SHA" =~ ^[0-9a-f]{40}$ ]]
[[ "$BENCHMARK_BASE_SHA" != "$BENCHMARK_HEAD_SHA" && "$RUN_HEAD_SHA" == "$BENCHMARK_HEAD_SHA" ]]
[[ "$(git -C base rev-parse HEAD)" == "$BENCHMARK_BASE_SHA" ]]
[[ "$(git -C head rev-parse HEAD)" == "$BENCHMARK_HEAD_SHA" ]]
git -C head merge-base --is-ancestor "$BENCHMARK_BASE_SHA" "$BENCHMARK_HEAD_SHA"
