#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/vitest-project-extends/module-cache-build-the-actual-source-addon-and-vite-adapter.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
vp run --workspace-root build:native:test
vp run --workspace-root build:vite-plugin
node npm/builder/vite/scripts/run-tests.ts --require-prepared
