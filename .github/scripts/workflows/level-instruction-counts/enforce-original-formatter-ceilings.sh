#!/usr/bin/env bash
set -euo pipefail
BASE_ARGS=()
if [[ -f "${RUNNER_TEMP}/base-formatter-instruction-budgets.toml" ]]; then
  BASE_ARGS=(--base-budgets "${RUNNER_TEMP}/base-formatter-instruction-budgets.toml")
fi
node tools/benchmarks/scripts/instruction-counts.mjs --formatter --check \
  --measurement "${RUNNER_TEMP}/formatter-instruction-counts/measurement.json" "${BASE_ARGS[@]}"
