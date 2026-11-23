#!/usr/bin/env bash
set -euo pipefail
mapfile -t archives < <(find "$RUNNER_TEMP/symbol-replay-download" -type f -name '*.tgz')
[[ "${#archives[@]}" == 1 ]]
restore=/home/runner/_work/_temp
[[ ! -e "$restore/cold-scale" ]]
mkdir -p "$restore"
tar -xzf "${archives[0]}" -C "$restore" \
  cold-scale/record-head/direct-process/perf.data \
  cold-scale/record-head/direct-process/receipt.json \
  cold-scale/binaries/normal-head/vize \
  cold-scale/builds/normal-head.json
node tools/benchmarks/scripts/source-symbolizer-symbol-cli.ts \
  --perf "$SMOKE_PERF" --root "$restore/cold-scale" \
  --out "$RUNNER_TEMP/symbol-replay"
