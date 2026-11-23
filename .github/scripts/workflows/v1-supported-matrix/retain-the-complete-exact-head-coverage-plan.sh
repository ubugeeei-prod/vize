#!/usr/bin/env bash
mkdir -p .qualification
node tools/commands/release/v1-exact-candidate.ts plan \
  --candidate "$SOURCE_SHA" \
  --output .qualification/v1-exact-candidate-plan.json
