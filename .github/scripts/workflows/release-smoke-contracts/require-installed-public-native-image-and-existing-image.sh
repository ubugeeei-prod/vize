#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/release-smoke-contracts/require-installed-public-native-image-and-existing-image.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
node --test \
  tests/tooling/release/release-smoke-public-native.test.ts \
  tests/tooling/native-build-token.test.ts \
  tests/tooling/native-binding-identity.test.ts
