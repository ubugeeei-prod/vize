#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/turbopack-consumers/install-consumer-dependencies.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
vp install \
  --filter vize-workspace \
  --filter '@vize/tests' \
  --filter '@vizejs/rspack-plugin...' \
  --filter '@vizejs/turbopack-plugin...' \
  --frozen-lockfile \
  --prefer-offline
