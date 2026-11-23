#!/usr/bin/env bash
echo "matrix=$(node tools/commands/ci/github/app-e2e-plan.ts --profile readiness)" >> "$GITHUB_OUTPUT" && echo "count=$(node tools/commands/ci/github/app-e2e-plan.ts --profile readiness --field count)" >> "$GITHUB_OUTPUT"
