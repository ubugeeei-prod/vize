#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/release-glibc-floor/prepare-the-native-platform-package.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
node tools/support/github/stage-built-native-packages.ts npm/native
test -f "npm/native/npm/$NATIVE_PLATFORM/vize-vitrine.$NATIVE_PLATFORM.node"
