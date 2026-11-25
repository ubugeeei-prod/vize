#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/babel-plugin-hosts/qualify-eight-supported-client-ssr-and-hydration-rows.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
set -euo pipefail
export VIZE_BABEL_PACKED_PLUGIN=$(find "$RUNNER_TEMP/babel-plugin/packed" -maxdepth 1 -name '*.tgz' -print -quit)
node npm/native/scripts/test-preparation.mjs node --test npm/builder/babel/test/runtime-renderers.test.ts
