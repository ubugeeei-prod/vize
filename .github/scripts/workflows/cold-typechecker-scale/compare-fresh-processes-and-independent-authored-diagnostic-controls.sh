#!/usr/bin/env bash
bench_cpu_set=$(node "$VIZE_CI_FILES/../shared/runner-resources.ts" first 8)
for round in 1 2 3; do
  order='base head'
  if [[ "$round" == 2 ]]; then order='head base'; fi
  for source in $order; do
    taskset -c "$bench_cpu_set" node tools/benchmarks/scripts/cold-typechecker-scale.ts --provider "$RUNNER_TEMP/provider" --files "$VIZE_BENCH_FILES" --tool cli --binary "$RUNNER_TEMP/cold-scale/binaries/normal-$source/vize" --out "$RUNNER_TEMP/cold-scale/$source-$round"
  done
done
for tool in mapper vue-tsc; do
  taskset -c "$bench_cpu_set" node tools/benchmarks/scripts/cold-typechecker-scale.ts --provider "$RUNNER_TEMP/provider" --files "$VIZE_BENCH_FILES" --tool "$tool" --out "$RUNNER_TEMP/cold-scale/$tool"
done
for tool in cli mapper vue-tsc; do
  args=()
  if [[ "$tool" == cli ]]; then args+=(--binary "$RUNNER_TEMP/cold-scale/binaries/normal-head/vize"); fi
  taskset -c "$bench_cpu_set" node tools/benchmarks/scripts/cold-typechecker-scale.ts --provider "$RUNNER_TEMP/provider" --files "$VIZE_BENCH_FILES" --tool "$tool" --invalid "${args[@]}" --out "$RUNNER_TEMP/cold-scale/control-$tool"
done
