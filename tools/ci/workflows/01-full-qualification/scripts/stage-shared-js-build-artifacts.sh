#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/ci/workflows/01-full-qualification/scripts/stage-shared-js-build-artifacts.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
mkdir -p .artifacts/shared-js-build/npm/native
mkdir -p .artifacts/shared-js-build/npm/cli
mkdir -p .artifacts/shared-js-build/npm/builder/vite
cp npm/native/*.node .artifacts/shared-js-build/npm/native/
cp npm/native/index.js npm/native/index.d.ts .artifacts/shared-js-build/npm/native/
cp -R npm/cli/dist .artifacts/shared-js-build/npm/cli/
cp -R npm/builder/vite/dist .artifacts/shared-js-build/npm/builder/vite/
