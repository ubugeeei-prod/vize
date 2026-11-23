#!/usr/bin/env bash
node head/tools/benchmarks/scripts/doctor-filter-paired.ts \
  "$GITHUB_WORKSPACE/base/target/ci-opt/examples/filter_patterns_bench" \
  "$GITHUB_WORKSPACE/head/target/ci-opt/examples/filter_patterns_bench" \
  "$GITHUB_WORKSPACE/doctor-filter-bench.json"
