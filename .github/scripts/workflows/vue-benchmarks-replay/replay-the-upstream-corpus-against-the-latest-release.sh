#!/usr/bin/env bash
node tools/benchmarks/scripts/vue-benchmarks-replay.ts \
  --vize-bin "$RUNNER_TEMP/vize-release/node_modules/.bin/vize" \
  --expect tools/benchmarks/scripts/vue-benchmarks-replay-expect-release.json \
  --json replay-release-results.json
