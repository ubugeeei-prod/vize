#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/ci/workflows/08-nuxt-styles/scripts/select-changed-product-inputs.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
if [[ "$BASE_SHA" =~ ^[a-f0-9]{40}$ && ! "$BASE_SHA" =~ ^0+$ ]]; then
  git fetch --no-tags --depth=1 origin "$BASE_SHA"
fi
node tools/support/compat/github/plan-product-integrations.ts "$GITHUB_EVENT_NAME" "$BASE_SHA" "$GITHUB_SHA"
