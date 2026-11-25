#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/ci/workflows/06-native-smoke/scripts/install-package-dependencies.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
vp install --frozen-lockfile --prefer-offline \
  --filter . \
  --filter '@vize/tools' \
  --filter './npm/native...' \
  --filter './npm/cli...' \
  --filter './npm/builder/vite...'
