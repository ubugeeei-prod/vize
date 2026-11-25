#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/ci/workflows/06-native-smoke/scripts/fresh-install-and-runtime-smoke-published-tarballs.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
node tools/commands/release/npm/smoke-release-install.ts --prepare-manifests --runtime-checks --bun-runtime-checks \
  npm/native npm/native/npm/* \
  npm/cli npm/builder/vite
