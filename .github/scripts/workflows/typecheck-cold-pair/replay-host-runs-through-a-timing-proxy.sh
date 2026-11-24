#!/usr/bin/env bash
set -euo pipefail
hosts=tools/benchmarks/typecheck-hosts/node_modules/.bin
mappers=(--mapper "base=replay-bin/cli/base/vize" --mapper "head=replay-bin/cli/head/vize")
node tools/benchmarks/scripts/content-mapper-replay-hosts.ts --work "$RUNNER_TEMP/replay" \
  --corpora "$CORPORA" --tsc "$hosts/tsc" --tsrs "$hosts/tsrs" "${mappers[@]}" --ready \
  --runs "$HOST_RUNS" --out "$RUNNER_TEMP/replay-out" > /dev/null
cat "$RUNNER_TEMP/replay-out/hosts.md" >> "$GITHUB_STEP_SUMMARY"
