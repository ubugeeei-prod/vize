#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/level-instruction-counts/enforce-pinned-ceilings.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
set -euo pipefail
BASE_ARGS=()
if [[ -f "${RUNNER_TEMP}/base-instruction-budgets.toml" ]]; then
  BASE_ARGS=(--base-budgets "${RUNNER_TEMP}/base-instruction-budgets.toml")
fi
node tools/benchmarks/scripts/instruction-counts.mjs --check \
  --measurement "${RUNNER_TEMP}/instruction-counts/measurement.json" "${BASE_ARGS[@]}"
