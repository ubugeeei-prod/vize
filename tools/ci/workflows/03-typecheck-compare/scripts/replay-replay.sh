#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/ci/workflows/03-typecheck-compare/scripts/replay-replay.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
set -euo pipefail
read -ra streams < "$RUNNER_TEMP/streams.args"
mappers=(--mapper "base=replay-bin/cli/base/vize" --mapper "head=replay-bin/cli/head/vize")
if [[ "$LAUNCHER" == true ]]; then
  mappers+=(--mapper "base-npm=node:$RUNNER_TEMP/npm-base/npm/cli/bin/vize")
  mappers+=(--mapper "head-npm=node:$RUNNER_TEMP/npm-head/npm/cli/bin/vize")
fi
node tools/benchmarks/scripts/content-mapper-replay.ts "${streams[@]}" "${mappers[@]}" \
  --ready --runs "$RUNS" $ALLOW --out "$RUNNER_TEMP/replay-out" > /dev/null
{
  echo "## Content mapper replay: head \`${HEAD::12}\` vs base \`${BASE::12}\` ($(nproc) vCPUs)"
  echo ""
  echo "Wall is the whole mapper process (spawn to exit) for burst, and request write to reply for serial. CPU is user+sys of the mapper process tree."
  echo ""
  cat "$RUNNER_TEMP/replay-out/replay.md"
} >> "$GITHUB_STEP_SUMMARY"
