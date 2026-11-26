#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/ci/workflows/01-check/scripts/check-js-ts.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
vp run --no-cache --filter './npm/native' build:debug && vp run --workspace-root check:ci
