#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/release-smoke-contracts/require-the-stable-node-release-smoke-contracts.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
node --test \
  tests/tooling/release/release-smoke-parity-peer-copy.test.ts \
  tests/tooling/release/release-smoke-install.test.ts \
  tests/tooling/release/release-smoke-runtime-oracle.test.ts \
  tests/tooling/release/release-smoke-typescript-runtime.test.ts
