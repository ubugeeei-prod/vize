#!/usr/bin/env bash
node head/tools/benchmarks/scripts/type-snapshot-paired.ts \
  "$GITHUB_WORKSPACE/base/target/ci-opt/examples/type_snapshot_bench" \
  "$GITHUB_WORKSPACE/head/target/ci-opt/examples/type_snapshot_bench" \
  "$GITHUB_WORKSPACE/type-snapshot-bench.json"
