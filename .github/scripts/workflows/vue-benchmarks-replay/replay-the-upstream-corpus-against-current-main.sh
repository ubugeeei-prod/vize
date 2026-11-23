#!/usr/bin/env bash
node tools/benchmarks/scripts/vue-benchmarks-replay.ts \
  --vize-bin target/ci/vize \
  --expect tools/benchmarks/scripts/vue-benchmarks-replay-expect-main.json \
  --json replay-main-results.json
