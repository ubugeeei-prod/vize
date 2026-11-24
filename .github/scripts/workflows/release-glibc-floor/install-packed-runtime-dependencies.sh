#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/release-glibc-floor/install-packed-runtime-dependencies.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
vp install --frozen-lockfile --prefer-offline \
  --filter . \
  --filter '@vize/tools' \
  --filter './npm/native...' \
  --filter './npm/cli...' \
  --filter './npm/builder/vite...'
