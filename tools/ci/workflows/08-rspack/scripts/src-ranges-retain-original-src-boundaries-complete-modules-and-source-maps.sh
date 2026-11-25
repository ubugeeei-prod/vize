#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/ci/workflows/08-rspack/scripts/src-ranges-retain-original-src-boundaries-complete-modules-and-source-maps.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
node npm/native/scripts/test-preparation.mjs node --test \
  npm/builder/rspack/src/shared/src-blocks.test.ts \
  npm/builder/rspack/src/shared/source-map.test.ts \
  npm/builder/rspack/src/loader/index.test.ts
