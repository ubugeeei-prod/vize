#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/ci/workflows/03-ci-walltime/scripts/build-the-same-native-packages-and-docs-once.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
node tools/support/github/run-many.ts vp run --filter './npm/native' build:ci -- vp run --filter './npm/cli' build -- vp run --filter './npm/builder/vite' build
vp run --no-cache --filter './docs' build
