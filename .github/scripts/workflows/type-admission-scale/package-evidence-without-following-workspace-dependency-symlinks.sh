#!/usr/bin/env bash
set -euo pipefail
mkdir -p "$RUNNER_TEMP/admission-scale"
git rev-parse HEAD > "$RUNNER_TEMP/admission-scale/source.txt"
node --version > "$RUNNER_TEMP/admission-scale/node.txt"
tar --exclude='*/node_modules' --exclude='*/xdg-cache' --exclude='*/tmp' \
  -czf "$RUNNER_TEMP/admission-scale-evidence.tgz" -C "$RUNNER_TEMP/admission-scale" .
