#!/usr/bin/env bash
node tools/support/github/stage-built-native-packages.ts npm/native
test -f "npm/native/npm/$NATIVE_PLATFORM/vize-vitrine.$NATIVE_PLATFORM.node"
