## Summary
With `@vizejs/nuxt`, page meta is dropped when `definePageMeta` is explicitly imported from `#imports`. The route has no `meta`, and `definePageMeta` stays as a setup binding. Auto-imported `definePageMeta` works. Similar to #2129 (explicit import from another module, closed).

## Repro (Nuxt 4.5.2, vue 3.5)
```ts
// nuxt.config.ts
export default defineNuxtConfig({
  ssr: false,
  modules: process.env.VIZE === "1" ? ["@vizejs/nuxt"] : [],
  vize: { compiler: true, lint: false, checker: false, musea: false },
});
```
```vue
<!-- app/app.vue -->
<template><NuxtLayout><NuxtPage /></NuxtLayout></template>
```
```vue
<!-- app/layouts/default.vue -->
<template><div><header id="default-header">DEFAULT LAYOUT</header><slot /></div></template>
```
```vue
<!-- app/layouts/bare.vue -->
<template><main id="bare-layout"><slot /></main></template>
```
```vue
<!-- app/pages/index.vue -->
<script setup lang="ts">
import { definePageMeta } from "#imports";
definePageMeta({ layout: "bare" });
</script>
<template><p>page</p></template>
```
`VIZE=1 nuxi generate` (also `nuxi dev`).

## Expected (without vize)
`<main id="bare-layout"><p>page</p></main>`; the route record has `meta: { layout: "bare" }`.

## Actual (`VIZE=1`)
`<div><header id="default-header">DEFAULT LAYOUT</header><p>page</p></div>`; no route meta, `definePageMeta` is kept as a setup binding.

## Notes
- Removing the import (relying on auto-import) fixes it.
- `imports.autoImport: false` and `defineOptions` make no difference.
- Other page meta (`middleware`, `redirect`, `validate`, …) presumably takes the same path.

## Version
@vizejs/nuxt 0.429.1, nuxt 4.5.2
Related: #2129

