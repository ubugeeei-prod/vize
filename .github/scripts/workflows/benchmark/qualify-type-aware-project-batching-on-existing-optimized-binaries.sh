#!/usr/bin/env bash
node --test head/tools/benchmarks/scripts/patina-project-batch.test.mjs
node head/tools/benchmarks/scripts/patina-project-batch.mjs \
  "$GITHUB_WORKSPACE/base/target/ci-opt/vize" \
  "$GITHUB_WORKSPACE/head/target/ci-opt/vize" \
  "$BENCHMARK_BASE_SHA" "$BENCHMARK_HEAD_SHA" \
  "$GITHUB_WORKSPACE/patina-project-batch"
