#!/usr/bin/env bash
test ! -s "$RUNNER_TEMP/frontend-bench/$BENCH_NAME.json" || node tools/support/github/rust-frontend-threads-bench.ts summarize "$RUNNER_TEMP/frontend-bench/$BENCH_NAME.json" >> "$GITHUB_STEP_SUMMARY"
