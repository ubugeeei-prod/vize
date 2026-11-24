#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/actions/app-e2e-row/scripts/build-local-vize-packages.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
node tools/support/github/run-many.ts \
  vp run --filter './npm/cli' build -- \
  vp run --filter './npm/builder/vite' build -- \
  vp run --filter './npm/framework/nuxt-lint-config' build -- \
  vp run --filter './npm/linter/oxlint' build -- \
  vp run --filter './npm/builder/vite-musea' build -- \
  vp run --filter './npm/framework/nuxt' build -- \
  vp run --filter './npm/framework/musea-nuxt' build
