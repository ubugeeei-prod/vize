#!/usr/bin/env bash
set -euo pipefail
binary=$(cat "$RUNNER_TEMP/component-casing/source-binary.path")
node tools/benchmarks/scripts/canon-component-casing.ts "$RUNNER_TEMP/component-casing-provider" "$binary" "$RUNNER_TEMP/component-casing/proof"
