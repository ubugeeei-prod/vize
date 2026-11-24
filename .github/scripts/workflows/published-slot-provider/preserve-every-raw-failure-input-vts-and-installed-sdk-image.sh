#!/usr/bin/env bash
set -euo pipefail
if [[ -d "$RUNNER_TEMP/slot-provider-public" ]]; then
  tar --exclude=node_modules --exclude=registry-installation -czf "$RUNNER_TEMP/slot-provider-public-raw.tgz" -C "$RUNNER_TEMP" slot-provider-public
fi
