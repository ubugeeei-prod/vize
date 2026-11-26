#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/ci/workflows/08-turbopack/scripts/build-both-public-adapters-with-bundled-shared-code.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
vp run --filter @vizejs/rspack-plugin build
vp run --filter @vizejs/turbopack-plugin build
