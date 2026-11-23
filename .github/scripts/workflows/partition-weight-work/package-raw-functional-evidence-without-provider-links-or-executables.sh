#!/usr/bin/env bash
set -euo pipefail
tar --exclude='*/node_modules' --exclude='*/project' -czf "$RUNNER_TEMP/partition-weight-raw.tgz" -C "$RUNNER_TEMP" partition-weight
