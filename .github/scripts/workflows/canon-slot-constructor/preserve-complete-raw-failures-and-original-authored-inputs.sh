#!/usr/bin/env bash
set -euo pipefail
if [[ -d "$RUNNER_TEMP/original27-current-evidence" ]]; then
  tar --exclude=source-vize --exclude=node_modules -czf "$RUNNER_TEMP/original27-current.tgz" -C "$RUNNER_TEMP" original27-current-evidence
fi
