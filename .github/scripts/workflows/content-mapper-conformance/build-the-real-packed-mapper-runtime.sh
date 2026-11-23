#!/usr/bin/env bash
vp run --filter './npm/native' build:ci
(cd npm/cli && vp pack)
node tools/support/github/stage-built-native-packages.ts npm/native
sha256sum npm/native/vize-vitrine.linux-x64-gnu.node > "$RUNNER_TEMP/official-css-source/native-image.sha256"
