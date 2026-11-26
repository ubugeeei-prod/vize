#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/ci/workflows/03-instruction-counts/scripts/enforce-original-formatter-ceilings.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
set -euo pipefail
BASE_ARGS=()
if [[ -f "${RUNNER_TEMP}/base-formatter-instruction-budgets.toml" ]]; then
  BASE_ARGS=(--base-budgets "${RUNNER_TEMP}/base-formatter-instruction-budgets.toml")
fi
node tools/benchmarks/scripts/instruction-counts.mjs --formatter --check \
  --measurement "${RUNNER_TEMP}/formatter-instruction-counts/measurement.json" "${BASE_ARGS[@]}"
