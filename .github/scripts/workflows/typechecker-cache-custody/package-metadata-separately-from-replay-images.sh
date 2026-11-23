#!/usr/bin/env bash
set -euo pipefail
mkdir -p "$RUNNER_TEMP/cache-evidence"
cp "$RUNNER_TEMP/cache-provider/package.json" "$RUNNER_TEMP/cache-provider/package-lock.json" "$RUNNER_TEMP/cache-evidence/" 2>/dev/null || true
tar --exclude='*/node_modules' -czf "$RUNNER_TEMP/cache-metadata.tgz" -C "$RUNNER_TEMP" cache-evidence
if [[ -f target/ci/vize ]]; then tar -czf "$RUNNER_TEMP/cache-source-image.tgz" target/ci/vize target/ci/vize.differential-build.json; fi
