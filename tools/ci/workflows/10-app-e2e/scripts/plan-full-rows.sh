#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/ci/workflows/10-app-e2e/scripts/plan-full-rows.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
echo "suite=$REQUESTED_SUITE" >> "$GITHUB_OUTPUT" && echo "matrix=$(node tools/ci/commands/github/app-e2e-plan.ts --profile full --suite "$REQUESTED_SUITE")" >> "$GITHUB_OUTPUT" && echo "count=$(node tools/ci/commands/github/app-e2e-plan.ts --profile full --suite "$REQUESTED_SUITE" --field count)" >> "$GITHUB_OUTPUT"
