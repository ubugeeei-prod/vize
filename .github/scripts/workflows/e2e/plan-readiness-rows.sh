#!/usr/bin/env bash
echo "matrix=$(node tools/ci/commands/github/app-e2e-plan.ts --profile readiness)" >> "$GITHUB_OUTPUT" && echo "count=$(node tools/ci/commands/github/app-e2e-plan.ts --profile readiness --field count)" >> "$GITHUB_OUTPUT"
