#!/usr/bin/env bash
set -euo pipefail
export VIZE_BENCH_CPU_SET=$(node tools/benchmarks/scripts/cold-typechecker-scale-affinity.ts)
taskset -c "$VIZE_BENCH_CPU_SET" node tools/benchmarks/scripts/cold-typechecker-scale-paired.ts --provider "$RUNNER_TEMP/provider" --out "$RUNNER_TEMP/cold-paired/observations" --base "$RUNNER_TEMP/cold-paired/builds/base.json" --head "$RUNNER_TEMP/cold-paired/builds/head.json"
