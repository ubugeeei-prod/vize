#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/cold-typechecker-scale/require-original-controls-and-full-output-before-eight-candidate-primari.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
set -euo pipefail
export VIZE_BENCH_CPU_SET=$(node tools/benchmarks/scripts/cold-typechecker-scale-affinity.ts)
taskset -c "$VIZE_BENCH_CPU_SET" node tools/benchmarks/scripts/cold-typechecker-scale-candidate-paired.ts --provider "$RUNNER_TEMP/provider" --out "$RUNNER_TEMP/cold-paired/observations" --case "$RUNNER_TEMP/cold-paired/selection/source-case.json" --base "$RUNNER_TEMP/cold-paired/builds/base.json" --head "$RUNNER_TEMP/cold-paired/builds/head.json" --allocation-base "$RUNNER_TEMP/cold-paired/allocation/base/build.json" --allocation-head "$RUNNER_TEMP/cold-paired/allocation/head/build.json" --custody-base "$RUNNER_TEMP/cold-paired/builds/custody-base.json" --custody-head "$RUNNER_TEMP/cold-paired/builds/custody-head.json"
