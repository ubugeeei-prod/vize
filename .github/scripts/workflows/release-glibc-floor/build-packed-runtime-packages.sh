#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/release-glibc-floor/build-packed-runtime-packages.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
vp run --filter './npm/cli' build
vp run --filter './npm/builder/unplugin' build
vp run --filter './npm/builder/vite' build
