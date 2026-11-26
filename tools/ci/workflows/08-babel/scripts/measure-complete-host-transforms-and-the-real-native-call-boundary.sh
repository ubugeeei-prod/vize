#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/ci/workflows/08-babel/scripts/measure-complete-host-transforms-and-the-real-native-call-boundary.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
set -euo pipefail
export VIZE_BABEL_PACKED_PLUGIN=$(find "$RUNNER_TEMP/babel-plugin/packed" -maxdepth 1 -name '*.tgz' -print -quit)
node npm/native/scripts/test-preparation.mjs node npm/builder/babel/test/profile-host.ts
