#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/pr-js-checks/verify-imported-type-projects-with-the-built-native-image.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
set -euo pipefail
shopt -s nullglob
bindings=("$GITHUB_WORKSPACE"/npm/native/*.node)
test "${#bindings[@]}" -eq 1
capture="$RUNNER_TEMP/imported-type-project"
mkdir -p "$capture"
node tests/native/imported-type-project-stats.ts "${bindings[0]}" "$capture/receipt.json"
