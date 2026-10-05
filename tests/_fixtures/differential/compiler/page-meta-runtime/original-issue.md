### Area

Compiler (SFC script), outside the Nuxt integration — follow-up to #7035

### Version

`@vizejs/native` / `@vizejs/vite-plugin` / `vize` 0.432.0

### Minimal reproduction

```js
import { compileSfc } from "@vizejs/native";

const src = `<script setup lang="ts">
import { definePageMeta } from "#imports";
definePageMeta({ layout: "plain" });
</script>
<template><div>page</div></template>
`;
console.log(compileSfc(src, { filename: "Page.vue" }).code);
```

Same with `vize build Page.vue --no-config`, and with the Vite plugin (no Nuxt module involved).

### Actual

Both the import and the call are removed:

```js
export default {
  __name: "Page",
  render: _sfc_render,
  setup(__props) {
    return {};
  }
};
```

A global (not imported) `definePageMeta(...)` is removed as well. An import from any other module (`import { definePageMeta } from "./meta"`) is kept.

### Expected

Outside Nuxt, keep the code as `@vue/compiler-sfc` does:

```js
import { definePageMeta } from "#imports";
export default /*@__PURE__*/_defineComponent({
  __name: 'Page',
  setup(__props) {
definePageMeta({ layout: "plain" });
…
```

`definePageMeta` is a Nuxt macro, and extracting it is the job of Nuxt's page-meta transform (which #7035 fixed for `@vizejs/nuxt`). When the plain compiler drops it, any non-Nuxt pipeline that compiles the same page breaks. Concretely, a Vitest component test that mocks `#imports` and asserts on `definePageMeta`'s argument (layout name, `middleware` function) passes with `@vitejs/plugin-vue` and fails with `@vizejs/vite-plugin`, because the call no longer exists.

Suggestion: only strip page macros when the Nuxt integration asks for it (e.g. an option set by `@vizejs/nuxt`), or provide an opt-out.

### Environment

- OS: macOS 26.6.2 (arm64)
- Node.js: 26.6.0
- Vue: 3.5.41

