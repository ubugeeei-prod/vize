#!/usr/bin/env bash
node npm/native/scripts/test-preparation.mjs node --test \
  npm/builder/rspack/src/shared/src-blocks.test.ts \
  npm/builder/rspack/src/shared/source-map.test.ts \
  npm/builder/rspack/src/loader/index.test.ts
