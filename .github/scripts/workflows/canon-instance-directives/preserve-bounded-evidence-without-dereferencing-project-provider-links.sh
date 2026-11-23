#!/usr/bin/env bash
set -euo pipefail
mkdir -p "$RUNNER_TEMP/instance-directives"
tar --exclude='*/node_modules' --exclude='*/.vize' --exclude='*/binaries' -czf "$RUNNER_TEMP/instance-directives-evidence.tgz" -C "$RUNNER_TEMP" instance-directives
