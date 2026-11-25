#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/ci/workflows/03-source-comparison/scripts/skip-benchmark-when-base-metadata-is-invalid.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
{
  echo "## PR Benchmark"
  echo
  echo "Base metadata could not be parsed, so the benchmark was skipped for this PR."
  echo
  echo "| status | reason |"
  echo "| --- | --- |"
  echo "| skipped | base checkout is not buildable |"
} > benchmark-summary.md
printf '{"skipped":true,"reason":"base_metadata_invalid","base":"%s","head":"%s"}\n' \
  "$BENCHMARK_BASE_SHA" \
  "$BENCHMARK_HEAD_SHA" \
  > benchmark-results.json
