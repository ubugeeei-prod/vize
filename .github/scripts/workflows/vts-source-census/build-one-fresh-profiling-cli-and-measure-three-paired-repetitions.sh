#!/usr/bin/env bash
set -euo pipefail
node tools/benchmarks/scripts/vts-source-census.ts \
  --out "$RUNNER_TEMP/census-evidence" \
  --target "$RUNNER_TEMP/census-target" \
  --provider "$RUNNER_TEMP/census-provider" \
  --runtime "/tmp/vize-vts-source-census-${GITHUB_RUN_ID}-${GITHUB_RUN_ATTEMPT}" \
  --source-sha "$SOURCE_SHA" \
  --source-parent "$SOURCE_PARENT" \
  --driver-sha "$GITHUB_WORKFLOW_SHA"
