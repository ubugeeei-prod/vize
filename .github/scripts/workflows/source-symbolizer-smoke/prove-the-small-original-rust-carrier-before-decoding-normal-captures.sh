#!/usr/bin/env bash
set -euo pipefail
out="$RUNNER_TEMP/source-symbolizer/$GITHUB_RUN_ID-$GITHUB_RUN_ATTEMPT"
node tools/benchmarks/scripts/source-symbolizer-smoke.ts \
  --perf "$SMOKE_PERF" --out "$out/carrier"
