#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/ci/workflows/01-v1-matrix/scripts/verify-the-exact-source-checkout.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
test "$(git rev-parse HEAD)" = "$SOURCE_SHA"
