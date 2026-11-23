#!/usr/bin/env bash
set -euo pipefail
mkdir -p "$RUNNER_TEMP/public-acceptance"
cp "$RUNNER_TEMP/public-provider/package.json" "$RUNNER_TEMP/public-provider/package-lock.json" "$RUNNER_TEMP/public-acceptance/" 2>/dev/null || true
tar --exclude='*/node_modules' -czf "$RUNNER_TEMP/public-acceptance.tgz" -C "$RUNNER_TEMP" public-acceptance
