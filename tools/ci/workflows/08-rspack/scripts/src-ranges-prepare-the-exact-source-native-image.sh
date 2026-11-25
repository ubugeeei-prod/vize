#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/ci/workflows/08-rspack/scripts/src-ranges-prepare-the-exact-source-native-image.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
test "$(git rev-parse HEAD)" = "$SOURCE_SHA"
vp run --workspace-root build:native:test
