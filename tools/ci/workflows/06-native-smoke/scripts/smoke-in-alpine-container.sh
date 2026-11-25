#!/bin/sh
# DO NOT EDIT. Generated from tools/ci/workflows/06-native-smoke/scripts/smoke-in-alpine-container.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
set -o pipefail
corepack enable
npm install --global --allow-scripts=bun bun@1.3.14 vite-plus@0.1.24
node tools/commands/release/npm/smoke-release-install.ts \
  --prepare-manifests --runtime-checks \
  npm/native "npm/native/npm/$NATIVE_PLATFORM" \
  npm/cli npm/builder/vite \
  2>&1 | tee /evidence/musl-smoke.log
grep -F "\"package\":\"$VIZE_SMOKE_EXPECTED_NATIVE_PACKAGE\"" \
  /evidence/musl-smoke.log
grep -F '"schema":"vize.loadedNativeBinding"' \
  /evidence/musl-smoke.log
