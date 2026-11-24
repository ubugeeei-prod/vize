#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/content-mapper-replay/count-allocations.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
set -euo pipefail
read -ra streams < "$RUNNER_TEMP/streams.args"
node tools/benchmarks/scripts/content-mapper-replay.ts "${streams[@]}" \
  --mapper "base=replay-bin/alloc/base/vize" --mapper "head=replay-bin/alloc/head/vize" \
  --ready --runs 0 --slowest 0 $ALLOW --out "$RUNNER_TEMP/replay-out/alloc" > /dev/null
{ echo "## Allocation counts (--features profiling, one burst)"; echo ""; cat "$RUNNER_TEMP/replay-out/alloc/replay.md"; } >> "$GITHUB_STEP_SUMMARY"
