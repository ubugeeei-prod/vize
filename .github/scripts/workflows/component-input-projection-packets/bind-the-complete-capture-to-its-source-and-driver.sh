#!/usr/bin/env bash
set -euo pipefail
[[ "$SOURCE_SHA" =~ ^[0-9a-f]{40}$ ]]
test "$(git rev-parse HEAD)" = "$SOURCE_SHA"
EVIDENCE="$RUNNER_TEMP/component-input-packets"
printf 'EVIDENCE=%s\n' "$EVIDENCE" >> "$GITHUB_ENV"
mkdir -p "$EVIDENCE"
git rev-parse HEAD 'HEAD^{tree}' > "$EVIDENCE/source.txt"
printf '%s\n' "$DRIVER_SHA" > "$EVIDENCE/driver.txt"
