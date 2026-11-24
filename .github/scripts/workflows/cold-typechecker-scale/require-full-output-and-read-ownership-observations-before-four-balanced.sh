#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/cold-typechecker-scale/require-full-output-and-read-ownership-observations-before-four-balanced.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
set -euo pipefail
export VIZE_BENCH_CPU_SET=$(node tools/benchmarks/scripts/cold-typechecker-scale-affinity.ts)
taskset -c "$VIZE_BENCH_CPU_SET" node tools/benchmarks/scripts/cold-typechecker-scale-paired.ts --provider "$RUNNER_TEMP/provider" --out "$RUNNER_TEMP/cold-paired/observations" --base "$RUNNER_TEMP/cold-paired/builds/base.json" --head "$RUNNER_TEMP/cold-paired/builds/head.json"
