#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/pr-source-checks/expand-the-source-planner-checkout.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
git sparse-checkout set --no-cone --stdin < .github/source-plan-sparse-checkout
