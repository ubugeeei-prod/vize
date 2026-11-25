#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/vue-benchmarks-replay/replay-the-upstream-corpus-against-the-latest-release.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
node tools/benchmarks/scripts/vue-benchmarks-replay.ts \
  --vize-bin "$RUNNER_TEMP/vize-release/node_modules/.bin/vize" \
  --expect tools/benchmarks/scripts/vue-benchmarks-replay-expect-release.json \
  --json replay-release-results.json
