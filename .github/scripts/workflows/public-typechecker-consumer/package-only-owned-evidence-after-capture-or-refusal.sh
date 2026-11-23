#!/usr/bin/env bash
set -euo pipefail
mkdir -p "$RUNNER_TEMP/public-q16-evidence"
tar -czf "$RUNNER_TEMP/public-q16-evidence.tgz" -C "$RUNNER_TEMP" public-q16-evidence
