#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/rspack-src-block-ranges/prepare-the-exact-source-native-image.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
test "$(git rev-parse HEAD)" = "$SOURCE_SHA"
vp run --workspace-root build:native:test
