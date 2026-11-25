#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/release/smoke-vizejs-wasm.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
node tools/commands/release/npm/smoke-wasm-package.ts npm/wasm
node tools/commands/release/npm/smoke-release-install.ts npm/wasm
