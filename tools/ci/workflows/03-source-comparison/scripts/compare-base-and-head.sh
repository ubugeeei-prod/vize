#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/ci/workflows/03-source-comparison/scripts/compare-base-and-head.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
node head/tools/benchmarks/scripts/compare-pr.ts \
  --input "$GITHUB_WORKSPACE/head/tools/benchmarks/scripts/__in__" \
  --base-bin "$GITHUB_WORKSPACE/base/target/ci-opt/vize" \
  --head-bin "$GITHUB_WORKSPACE/head/target/ci-opt/vize" \
  --base-label "$BENCHMARK_BASE_SHA" \
  --head-label "$BENCHMARK_HEAD_SHA" \
  --runs "$VIZE_BENCH_RUNS" \
  --warmups "$VIZE_BENCH_WARMUPS" \
  --threshold "$VIZE_BENCH_REGRESSION_THRESHOLD_PERCENT" \
  --out "$GITHUB_WORKSPACE/benchmark-summary.md" \
  --json "$GITHUB_WORKSPACE/benchmark-results.json"
