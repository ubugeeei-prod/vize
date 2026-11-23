#!/usr/bin/env bash
bench_cpu_set=$(node "$VIZE_CI_FILES/../shared/runner-resources.ts" first 8)
for source in base head; do
  taskset -c "$bench_cpu_set" node tools/benchmarks/scripts/cold-typechecker-scale.ts --provider "$RUNNER_TEMP/provider" --files "$VIZE_BENCH_FILES" --tool profile --binary "$RUNNER_TEMP/vize-profile-$source" --out "$RUNNER_TEMP/cold-scale/profile-$source"
done
node "$VIZE_CI_FILES/capture-paired-rust-allocation-spans-without-cpu-sampling-overhead.ts"
