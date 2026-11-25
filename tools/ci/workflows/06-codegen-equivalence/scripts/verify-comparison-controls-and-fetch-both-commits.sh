#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/ci/workflows/06-codegen-equivalence/scripts/verify-comparison-controls-and-fetch-both-commits.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
set -euo pipefail
VIZE_CODEGEN_PROBE=1 node --test tests/tooling/levels-codegen-equivalence.test.ts
[[ "$BASE_SHA" =~ ^[0-9a-f]{40}$ && "$HEAD_SHA" =~ ^[0-9a-f]{40}$ ]]
git fetch --no-tags --depth=1 origin "$BASE_SHA" "$HEAD_SHA"
