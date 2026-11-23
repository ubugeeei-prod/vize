#!/usr/bin/env bash
set -euo pipefail
mkdir -p "$RUNNER_TEMP/out"
for layout in $LAYOUTS; do
  [[ "$layout" =~ ^(root|nested)$ ]]
  node editors/vscode/test/run-startup-latency.ts generate --root "$RUNNER_TEMP/ws-$layout" \
    --layout "$layout" --sfcs "$SFCS" --noise "$NOISE"
  xvfb-run -a node editors/vscode/test/run-startup-latency.ts run --code "$CODE_EXE" \
    --server "$RUNNER_TEMP/server/vize" --workspace "$RUNNER_TEMP/ws-$layout" \
    --extension "base=$PWD/ext-builds/base" --extension "head=$PWD/ext-builds/head" \
    --rounds "$ROUNDS" --profiles "$RUNNER_TEMP/profiles" --out "$RUNNER_TEMP/out/linux-$layout" \
    --title "Linux ($(nproc) vCPUs), $layout layout, $SFCS SFCs + $NOISE files" >> "$GITHUB_STEP_SUMMARY"
done
