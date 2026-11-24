#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/vue-benchmarks-replay/replay-the-upstream-corpus-against-current-main.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
node tools/benchmarks/scripts/vue-benchmarks-replay.ts \
  --vize-bin target/ci/vize \
  --expect tools/benchmarks/scripts/vue-benchmarks-replay-expect-main.json \
  --json replay-main-results.json
