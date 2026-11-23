#!/usr/bin/env bash
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
