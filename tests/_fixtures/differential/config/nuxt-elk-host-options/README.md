# Elk setupSFC host Vue options

`resolved-vite-config.ts` is an authored resolved-input reproduction, assembled
from the pinned Elk configuration and the dependency versions installed in
[App E2E run 38866347055](https://github.com/ubugeeei-prod/vize/actions/runs/38866347055).
It is not a runtime configuration dump or a replacement for running Elk.

The unchanged Elk gitlink is
[`8aa2b4650760eeba4ea1fdd9dc18d8a88ca5d7f0`](https://github.com/elk-zone/elk/blob/8aa2b4650760eeba4ea1fdd9dc18d8a88ca5d7f0/nuxt.config.ts).
It enables `@vue-macros/nuxt`, `macros.setupSFC`, and `vue.propsDestructure`.

- `@nuxt/schema@4.4.7`, `dist/index.mjs`: the root `vue` defaults provide
  `compilerOptions` and `transformAssetUrls`; `vite.vue` provides
  `script.hoistStatic`, `isProduction`, `template`, and `features.propsDestructure`.
- `@vue-macros/nuxt@3.1.2`, `dist/index.js`: setupSFC defaults `vite.vue.include`
  to `[/\.vue$/]`, normalizes a scalar to an array, and appends `REGEX_SETUP_SFC`.
- `@vue-macros/common@3.1.2`, `dist/index.js`: the appended expression is
  `/\.setup\.[cm]?[jt]sx?$/`.
- `@nuxt/vite-builder@4.4.7`, `dist/index.mjs`: both client and server builders
  pass the original `config.vue` into `@vitejs/plugin-vue`. Vize must preserve
  that object and its filter and script option identities during projection.

`provenance.json` identifies those independently inspected public package
sources and their complete-file SHA-256 values. `original-failure.log` preserves
the first failing development stack from the run. Production preview failed at
the same projector; VRT could not connect because the server never started.

`tests/tooling/vue-config-nuxt-host-context.test.ts` consumes this fixture and
the complete `projected.json` oracle for development, client build, and server
build. The oracle contains only the native defaults: host options must not leak
into native project settings. Separate controls retain scalar and array
include/exclude identities, private-context admission, and strict refusal of
mixed canonical or unknown options. No upstream configuration, test deadline,
or visual snapshot is changed by this fixture.

Full acceptance still requires the existing Elk preview, dev, and VRT Actions
jobs against the unchanged pinned consumer.
