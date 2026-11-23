#!/usr/bin/env bash
read -ra js <<< "$RELEASE_JS_PACKAGES"
node tools/commands/release/npm/smoke-release-install.ts --prepare-manifests --runtime-checks --bun-runtime-checks \
  npm/native npm/native/npm/* npm/fresco-native "${js[@]}"
