#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/level-dependencies/fetch-dependency-policy-comparison-base.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
dependency_base="$BASE_SHA"
if [[ -z "$dependency_base" ]]; then
  dependency_base="$(git rev-parse HEAD^)"
fi
if [[ ! "$dependency_base" =~ ^[0-9a-f]{40}$ || "$dependency_base" =~ ^0+$ ]]; then
  echo 'Missing dependency policy comparison base' >&2
  exit 1
fi
git fetch --no-tags --depth=1 origin "$dependency_base"
echo "DEPENDENCY_BASE=$dependency_base" >> "$GITHUB_ENV"
