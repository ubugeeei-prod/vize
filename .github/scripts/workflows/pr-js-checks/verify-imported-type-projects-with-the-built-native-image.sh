#!/usr/bin/env bash
set -euo pipefail
shopt -s nullglob
bindings=("$GITHUB_WORKSPACE"/npm/native/*.node)
test "${#bindings[@]}" -eq 1
capture="$RUNNER_TEMP/imported-type-project"
mkdir -p "$capture"
node tests/native/imported-type-project-stats.ts "${bindings[0]}" "$capture/receipt.json"
