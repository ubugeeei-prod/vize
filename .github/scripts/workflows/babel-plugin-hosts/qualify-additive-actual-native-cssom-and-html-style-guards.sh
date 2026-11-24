#!/usr/bin/env bash
set -euo pipefail
export VIZE_BABEL_PACKED_PLUGIN=$(find "$RUNNER_TEMP/babel-plugin/packed" -maxdepth 1 -name '*.tgz' -print -quit)
node npm/native/scripts/test-preparation.mjs node --test npm/builder/babel/test/style-guards.test.ts
