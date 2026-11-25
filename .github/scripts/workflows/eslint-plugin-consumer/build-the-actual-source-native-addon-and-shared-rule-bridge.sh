#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/eslint-plugin-consumer/build-the-actual-source-native-addon-and-shared-rule-bridge.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
vp run --filter './npm/native' build:ci
vp run --filter './npm/linter/oxlint' build
vp run --filter './npm/linter/eslint' build
