#!/usr/bin/env bash
mkdir -p target/app-e2e-plan && node tools/commands/ci/github/app-e2e-plan.ts --profile readiness --field evidence > target/app-e2e-plan/readiness.json
