#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/musea-browser/build-the-exact-source-native-binding-and-gallery.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
vp run --filter './npm/native' build:ci
vp run --filter './npm/cli' build
vp run --filter './npm/builder/vite' build
vp run --filter './npm/builder/vite-musea' build
