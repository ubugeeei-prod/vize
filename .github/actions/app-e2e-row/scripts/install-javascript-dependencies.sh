#!/usr/bin/env bash
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
