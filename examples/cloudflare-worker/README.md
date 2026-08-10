# Vize on Cloudflare Workers

This example compiles Vue single-file components inside Cloudflare Workers with the focused
`@vizejs/wasm` workerd artifact. The Worker follows the same lazy initialization pattern as
[`oxc-wasip1-workers`](https://github.com/Boshen/oxc-wasip1-workers): Wrangler imports the `.wasm`
file as a precompiled module, and the request handler initializes it once and reuses the promise for
warm requests.

## Run

Build the local `@vizejs/wasm` package first, then check the Worker bundle:

```bash
moon run --target native tools/moon/cmd/build_vize_wasm_package --
vp install
vp run --filter vize-cloudflare-worker-example check
```

Use Cloudflare's remote development runtime or deploy after authenticating Wrangler:

```bash
vp run --filter vize-cloudflare-worker-example dev
vp run --filter vize-cloudflare-worker-example deploy
```

The `check` command runs handler unit tests, starts the local workerd runtime for a real GET/POST
smoke test, and verifies the compressed Wrangler bundle. `GET /` compiles the built-in
`Counter.vue`. To compile another SFC:

```bash
curl -X POST http://localhost:8787 \
  -H 'content-type: application/json' \
  --data '{"source":"<template><main>{{ message }}</main></template><script setup lang=\"ts\">const message: string = \"Hello\"</script>","options":{"filename":"App.vue"}}'
```

The dedicated artifact omits browser-package linting, formatting, cross-file analysis, Musea, and
inspector exports. That keeps the compressed Worker below Cloudflare Workers' 3 MB free-plan limit
while exposing template, SFC, CSS, SSR, and Vapor compilation APIs through `instantiate()`.
