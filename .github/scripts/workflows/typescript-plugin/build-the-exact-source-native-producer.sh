#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/typescript-plugin/build-the-exact-source-native-producer.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
export NODE_OPTIONS="${NODE_OPTIONS:-} --import=file://$GITHUB_WORKSPACE/tools/support/ci/cargo-metadata-observer.ts?capture=$RUNNER_TEMP/typescript-plugin-cargo-metadata"
vp run --filter './npm/native' build:ci
