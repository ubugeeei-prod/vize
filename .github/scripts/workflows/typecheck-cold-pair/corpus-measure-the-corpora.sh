#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/typecheck-cold-pair/corpus-measure-the-corpora.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
set -euo pipefail
if [[ "$OWNED_PAIR" == true ]]; then
  workload_args=()
  if [[ -n "$OWNED_WORKLOADS" ]]; then workload_args+=(--workloads "$OWNED_WORKLOADS"); fi
  if [[ -n "$OWNED_LANES" ]]; then workload_args+=(--lanes "$OWNED_LANES"); fi
  node tools/benchmarks/scripts/typecheck-owned-pair.ts \
    --base "$RUNNER_TEMP/pair-base" --head "$RUNNER_TEMP/pair-head" \
    --base-sha "$BASE_SHA" --head-sha "$HEAD_SHA" \
    --work "$RUNNER_TEMP/pair-work" --out "$RUNNER_TEMP/pair-out" \
    "${workload_args[@]}"
  exit
fi
extra=(--corpora "$CORPORA")
if [[ "$BATCH" == true ]]; then extra+=(--batch); fi
if [[ "$FLOORS" == true ]]; then extra+=(--floors); fi
node tools/benchmarks/scripts/typecheck-corpus-pair.ts \
  --base "$RUNNER_TEMP/pair-base" --head "$RUNNER_TEMP/pair-head" \
  --base-sha "$BASE_SHA" --head-sha "$HEAD_SHA" --runs "$RUNS" \
  --work "$RUNNER_TEMP/pair-work" --out "$RUNNER_TEMP/pair-out" "${extra[@]}"
