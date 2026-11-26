#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/ci/workflows/01-pr-source/scripts/build-playground-dependencies.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
vp run --filter './npm/native' build:ci && vp run --workspace-root build:packages
