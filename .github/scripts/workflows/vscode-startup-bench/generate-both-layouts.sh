#!/usr/bin/env bash
set -euo pipefail
for layout in root nested; do
  node editors/vscode/test/run-startup-latency.ts generate --root "$RUNNER_TEMP/ws-$layout" \
    --layout "$layout" --sfcs 20 --noise 50 --node-modules 20
  test -f "$RUNNER_TEMP/ws-$layout/$(cat "$RUNNER_TEMP/ws-$layout/bench-target.txt")"
done
test -f "$RUNNER_TEMP/ws-root/tsconfig.json" && test ! -f "$RUNNER_TEMP/ws-nested/tsconfig.json"
node --check editors/vscode/test/suite/startup-latency.cjs
