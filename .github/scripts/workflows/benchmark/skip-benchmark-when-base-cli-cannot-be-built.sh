#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/benchmark/skip-benchmark-when-base-cli-cannot-be-built.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
{
  echo "## PR Benchmark"
  echo
  echo "Base CLI could not be built, so the benchmark was skipped for this PR."
  echo
  echo "| status | reason |"
  echo "| --- | --- |"
  echo "| skipped | base checkout does not compile |"
} > benchmark-summary.md
printf '{"skipped":true,"reason":"base_not_buildable","base":"%s","head":"%s"}\n' \
  "$BENCHMARK_BASE_SHA" \
  "$BENCHMARK_HEAD_SHA" \
  > benchmark-results.json
