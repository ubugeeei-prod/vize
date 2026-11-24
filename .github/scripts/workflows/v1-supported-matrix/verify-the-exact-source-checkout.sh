#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/v1-supported-matrix/verify-the-exact-source-checkout.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
test "$(git rev-parse HEAD)" = "$SOURCE_SHA"
