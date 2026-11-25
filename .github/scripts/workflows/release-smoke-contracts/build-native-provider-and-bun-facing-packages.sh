#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/release-smoke-contracts/build-native-provider-and-bun-facing-packages.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
vp run --filter './npm/native' build:ci
node tools/support/github/stage-built-native-packages.ts npm/native
vp run --filter './npm/cli' build
vp run --filter './npm/builder/vite' build
vp run --filter './npm/linter/oxlint' build
