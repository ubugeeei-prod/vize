#!/usr/bin/env bash
set -euo pipefail
if [[ -d "$RUNNER_TEMP/original27-public" ]]; then
  tar --exclude=node_modules --exclude=registry-installation -czf "$RUNNER_TEMP/original27-public-raw.tgz" -C "$RUNNER_TEMP" original27-public
fi
