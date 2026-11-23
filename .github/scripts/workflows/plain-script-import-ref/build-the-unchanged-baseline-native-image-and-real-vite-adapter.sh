#!/usr/bin/env bash
test "$(git rev-parse HEAD)" = "$SOURCE_SHA"
vp install --frozen-lockfile --prefer-offline --filter './npm/builder/vite...' --filter './npm/cli...' --filter './npm/native...'
vp run --workspace-root build:native:test
vp run --workspace-root build:vite-plugin
GITHUB_SHA="$SOURCE_SHA" node npm/native/scripts/native-build-receipt.mjs
git diff --exit-code -- crates davinci vendor npm/builder/vite/src
