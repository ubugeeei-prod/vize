#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/ci/workflows/06-native-smoke/scripts/build-npm-runtime-packages.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
vp run --filter './npm/cli' build
vp run --filter './npm/builder/vite' build
