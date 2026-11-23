#!/usr/bin/env bash
set -euo pipefail
cd "$RUNNER_TEMP"
tar --exclude=project --exclude=node_modules --exclude=tmp --exclude=cache -czf slot-constructor-evidence.tgz slot-evidence
