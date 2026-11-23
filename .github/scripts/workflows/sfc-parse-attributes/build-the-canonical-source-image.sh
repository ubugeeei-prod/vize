#!/usr/bin/env bash
set -euo pipefail
source_dir="$GITHUB_WORKSPACE"
if [ "$SOURCE_ROLE" = base ]; then source_dir="$GITHUB_WORKSPACE/.attrs-base"; fi
test "$(git -C "$source_dir" rev-parse HEAD)" = "$VIZE_SFC_ATTRS_SOURCE_SHA"
cd "$source_dir"
if [ "$SOURCE_ROLE" = head ]; then
  vp install --frozen-lockfile --prefer-offline
else
  vp install --filter './npm/native...' --frozen-lockfile --prefer-offline
fi
vp run --filter @vizejs/native build:ci
