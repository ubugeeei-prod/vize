#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/ci/workflows/10-app-e2e/scripts/verify-producer-sha.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
test "$(git rev-parse HEAD)" = "$EXPECTED_SHA"
