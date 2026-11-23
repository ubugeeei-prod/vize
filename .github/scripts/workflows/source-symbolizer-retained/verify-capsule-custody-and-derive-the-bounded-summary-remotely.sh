#!/usr/bin/env bash
set -euo pipefail
mkdir "$RUNNER_TEMP/metadata-summary"
node tools/benchmarks/scripts/cold-evidence-summary.ts \
  "$RUNNER_TEMP/thin-download" "$RUNNER_TEMP/metadata-summary/summary.json"
cp "$RUNNER_TEMP/source-artifact-api.json" "$RUNNER_TEMP/metadata-summary/capsule-api.json"
