#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/ci/workflows/01-v1-matrix/scripts/retain-the-complete-exact-head-coverage-plan.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
mkdir -p .qualification
node tools/commands/release/v1-exact-candidate.ts plan \
  --candidate "$SOURCE_SHA" \
  --output .qualification/v1-exact-candidate-plan.json
