#!/usr/bin/env bash
node tools/support/github/rust-frontend-threads-bench.ts summarize "$RUNNER_TEMP"/frontend-bench/*.json >> "$GITHUB_STEP_SUMMARY"
