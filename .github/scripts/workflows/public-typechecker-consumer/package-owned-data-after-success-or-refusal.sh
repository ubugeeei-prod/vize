#!/usr/bin/env bash
set -euo pipefail
mkdir -p "$RUNNER_TEMP/public-11339-evidence"
tar -czf "$RUNNER_TEMP/public-11339-evidence.tgz" -C "$RUNNER_TEMP" public-11339-evidence
