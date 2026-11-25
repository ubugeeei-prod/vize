#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/ci/workflows/10-app-e2e/scripts/write-readiness-plan-evidence.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
mkdir -p target/app-e2e-plan && node tools/ci/commands/github/app-e2e-plan.ts --profile readiness --field evidence > target/app-e2e-plan/readiness.json
