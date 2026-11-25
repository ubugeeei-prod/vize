#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/tool-benchmark/compare-vize-with-existing-tools.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
# Restore benchmark metadata label with:
node tools/benchmarks/scripts/compare-tools.mjs \
  --input tools/benchmarks/scripts/__in__ \
  --vize-bin target/release/vize \
  --runs "$VIZE_TOOL_BENCH_RUNS" \
  --warmups "$VIZE_TOOL_BENCH_WARMUPS" \
  --check-file-count "$VIZE_TOOL_BENCH_CHECK_FILE_COUNT" \
  --vite-file-count "$VIZE_TOOL_BENCH_VITE_FILE_COUNT" \
  --nuxt-file-count "$VIZE_TOOL_BENCH_NUXT_FILE_COUNT" \
  --musea-file-count "$VIZE_TOOL_BENCH_MUSEA_FILE_COUNT" \
  --large-blocks "$VIZE_TOOL_BENCH_LARGE_BLOCKS" \
  ${VIZE_TOOL_BENCH_TASKS:+--tasks "$VIZE_TOOL_BENCH_TASKS"} \
  --runner-label "blacksmith-32vcpu-ubuntu-2404" \
  --commit "${VIZE_BENCHMARK_REVISION}" \
  --ref "$VIZE_BENCH_REF" \
  --repository "${GITHUB_REPOSITORY}" \
  --run-url "${GITHUB_SERVER_URL}/${GITHUB_REPOSITORY}/actions/runs/${GITHUB_RUN_ID}" \
  --out tool-benchmark-summary.md \
  --json tool-benchmark-results.json \
  --doc performance-blacksmith.md
