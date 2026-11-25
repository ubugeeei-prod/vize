#!/usr/bin/env bash
node --test \
  npm/framework/nuxt/src/server-plugins.test.ts \
  npm/framework/nuxt/src/builder.test.ts \
  tests/tooling/nuxt5-dev-runtime.test.ts
