#!/usr/bin/env bash
node head/tools/benchmarks/scripts/type-snapshot-cli-paired.mjs \
  "$GITHUB_WORKSPACE/base/target/ci-opt/vize" \
  "$GITHUB_WORKSPACE/head/target/ci-opt/vize" \
  "$GITHUB_WORKSPACE/type-snapshot-cli-bench.json"
cat type-snapshot-cli-bench.json.md >> "$GITHUB_STEP_SUMMARY"
