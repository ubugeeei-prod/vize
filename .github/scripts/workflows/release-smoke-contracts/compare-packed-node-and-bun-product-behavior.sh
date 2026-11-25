#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/release-smoke-contracts/compare-packed-node-and-bun-product-behavior.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
node tools/commands/release/npm/smoke-release-install.ts \
  --prepare-manifests \
  --runtime-checks \
  --bun-runtime-checks npm/native npm/native/npm/* npm/cli npm/builder/vite npm/linter/oxlint
