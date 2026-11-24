#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/cold-typechecker-scale/capture-paired-rust-allocation-spans-without-cpu-sampling-overhead.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
bench_cpu_set=$(node "$VIZE_CI_FILES/../shared/runner-resources.ts" first 8)
for source in base head; do
  taskset -c "$bench_cpu_set" node tools/benchmarks/scripts/cold-typechecker-scale.ts --provider "$RUNNER_TEMP/provider" --files "$VIZE_BENCH_FILES" --tool profile --binary "$RUNNER_TEMP/vize-profile-$source" --out "$RUNNER_TEMP/cold-scale/profile-$source"
done
node "$VIZE_CI_FILES/capture-paired-rust-allocation-spans-without-cpu-sampling-overhead.ts"
