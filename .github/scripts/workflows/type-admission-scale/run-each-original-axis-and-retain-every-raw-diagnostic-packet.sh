#!/usr/bin/env bash
set -euo pipefail
node tools/benchmarks/scripts/type-admission-scale-suite.ts \
  --provider "$RUNNER_TEMP/admission-provider" --out "$RUNNER_TEMP/admission-scale" \
  --axis "$SCALE_AXIS" --sfcs "$SCALE_SFCS" --tool "$SCALE_TOOL"
