#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/prettier-plugin/build-exact-source-binding-and-package-exports.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
test "$(git rev-parse HEAD)" = "$SOURCE_SHA"
test "$(node "$VIZE_CI_FILES/build-exact-source-binding-and-package-exports.cjs")" = 24.14.0
vp run --workspace-root build:native:test
cd npm/formatter/prettier
vp check src test vite.config.ts
vp pack
