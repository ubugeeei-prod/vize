#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/rust-pr-target-seed/validate-the-complete-frozen-dependency-store-before-saving.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
vp install --frozen-lockfile --prefer-offline
git diff --exit-code -- package.json pnpm-lock.yaml pnpm-workspace.yaml
