#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/ci/workflows/01-pr-source/scripts/expand-the-source-planner-checkout.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
git sparse-checkout set --no-cone --stdin < .github/source-plan-sparse-checkout
