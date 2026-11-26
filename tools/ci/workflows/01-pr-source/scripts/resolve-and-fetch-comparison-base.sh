#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/ci/workflows/01-pr-source/scripts/resolve-and-fetch-comparison-base.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
comparison_base="$(node tools/support/compat/github/comparison-base.ts)"
if [[ ! "$comparison_base" =~ ^0+$ ]]; then
  # Only the two trees are compared, so the base needs no history.
  git fetch --no-tags --depth=1 origin "$comparison_base"
fi
