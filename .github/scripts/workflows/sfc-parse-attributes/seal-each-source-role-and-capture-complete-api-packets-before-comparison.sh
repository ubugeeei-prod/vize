#!/usr/bin/env bash
set -euo pipefail
source_dir="$GITHUB_WORKSPACE"
if [ "$SOURCE_ROLE" = base ]; then source_dir="$GITHUB_WORKSPACE/.attrs-base"; fi
node tests/native/sfc-parse-attributes-capture.mjs "$source_dir" "$SOURCE_ROLE" "$GITHUB_WORKSPACE/artifacts/$SOURCE_ROLE"
