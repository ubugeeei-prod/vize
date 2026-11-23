#!/usr/bin/env bash
mkdir -p "$RUNNER_TEMP/event-rename"
if [ -f target/ci/vize ]; then
  sha256sum target/ci/vize > "$RUNNER_TEMP/event-rename/cli.sha256"
fi
find "$RUNNER_TEMP/event-rename" -type f -print0 | sort -z | xargs -0 sha256sum > "$RUNNER_TEMP/event-rename-inventory.sha256"
