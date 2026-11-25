#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/ci/workflows/10-app-e2e/scripts/plan-readiness-rows.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
echo "matrix=$(node tools/ci/commands/github/app-e2e-plan.ts --profile readiness)" >> "$GITHUB_OUTPUT" && echo "count=$(node tools/ci/commands/github/app-e2e-plan.ts --profile readiness --field count)" >> "$GITHUB_OUTPUT"
