#!/usr/bin/env bash
set -euo pipefail
mkdir -p "$RUNNER_TEMP/census-evidence"
cp "$RUNNER_TEMP/census-provider/package.json" "$RUNNER_TEMP/census-provider/package-lock.json" "$RUNNER_TEMP/census-evidence/" 2>/dev/null || true
tar --exclude='*/node_modules' --exclude='*/binary' -czf "$RUNNER_TEMP/census-raw.tgz" -C "$RUNNER_TEMP" census-evidence
