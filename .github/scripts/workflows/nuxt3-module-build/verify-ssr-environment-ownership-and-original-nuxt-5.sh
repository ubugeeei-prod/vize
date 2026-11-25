#!/usr/bin/env bash
node --test \
  npm/framework/nuxt/src/ssr-runtime.test.ts \
  tests/tooling/nuxt5-runtime.test.ts \
  tests/tooling/nuxt-source-binding.test.mjs
