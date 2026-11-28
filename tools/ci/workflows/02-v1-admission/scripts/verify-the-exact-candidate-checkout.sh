#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/ci/workflows/02-v1-admission/scripts/verify-the-exact-candidate-checkout.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
[[ "$CANDIDATE_SHA" =~ ^[0-9a-f]{40}$ ]]
test "$(git rev-parse HEAD)" = "$CANDIDATE_SHA"
