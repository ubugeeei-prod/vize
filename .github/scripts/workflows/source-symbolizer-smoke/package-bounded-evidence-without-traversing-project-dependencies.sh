#!/usr/bin/env bash
set -euo pipefail
out="$RUNNER_TEMP/source-symbolizer/$GITHUB_RUN_ID-$GITHUB_RUN_ATTEMPT"
if [[ -d "$out" ]]; then
  tar -czf "$RUNNER_TEMP/source-symbolizer-evidence.tgz" \
    --exclude='*/project/node_modules' --exclude='*/tmp' --exclude='*/xdg-cache' \
    -C "$out" .
fi
