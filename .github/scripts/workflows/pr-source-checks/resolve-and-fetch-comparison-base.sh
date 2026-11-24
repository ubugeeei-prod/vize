#!/usr/bin/env bash
comparison_base="$(node tools/support/compat/github/comparison-base.ts)"
if [[ ! "$comparison_base" =~ ^0+$ ]]; then
  # Only the two trees are compared, so the base needs no history.
  git fetch --no-tags --depth=1 origin "$comparison_base"
fi
