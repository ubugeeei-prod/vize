#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/actions/app-e2e-row/scripts/install-javascript-dependencies.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
vp install --frozen-lockfile --prefer-offline \
  --filter './tests...' \
  --filter './npm/native...' \
  --filter './npm/cli...' \
  --filter './npm/builder/vite...' \
  --filter './npm/framework/nuxt-lint-config...' \
  --filter './npm/linter/oxlint...' \
  --filter './npm/framework/nuxt...' \
  --filter './npm/builder/vite-musea...' \
  --filter './npm/framework/musea-nuxt...'
