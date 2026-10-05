## Summary

With the Vize compiler on (`vize: { compiler: true }`), SFC entries in the client build manifest are keyed `pages/foo.vue?vue&vize` instead of `pages/foo.vue`. Nuxt's `build:manifest` hook removes page components (and global components) from the entry's `dynamicImports` by comparing against the plain source path (`pages/foo.vue`), so with the query nothing matches. Every page chunk, with its CSS and dependencies, then ends up as a `<link rel="prefetch">` in the generated HTML and is downloaded on the first visit.

Same family as #6897 (module ids ending in `.vue.ts` broke Nuxt's critical CSS).

## Reproduction

`package.json`: `nuxt@4.5.2`, `vue@3.5.42`, `vize@0.432.0`, `@vizejs/nuxt@0.432.0`

`nuxt.config.ts`

```ts
export default defineNuxtConfig({
  ssr: false,
  modules: ["@vizejs/nuxt"],
  vize: { compiler: process.env.VIZE_COMPILER === "1", lint: false, checker: false, musea: false }
});
```

`app/app.vue`: `<template><NuxtPage /></template>`

`app/pages/index.vue`: `<template><p>Home</p></template>`

`app/pages/reports.vue`

```vue
<script setup lang="ts">
import ReportTable from "../components/ReportTable.vue";
</script>

<template>
  <ReportTable />
</template>
```

`app/components/ReportTable.vue`: any component with a `<style scoped>` block.

```sh
VIZE_COMPILER=0 nuxt generate && grep -o 'rel="prefetch"' .output/public/index.html | wc -l   # 5
VIZE_COMPILER=1 nuxt generate && grep -o 'rel="prefetch"' .output/public/index.html | wc -l   # 8
```

The entry's `dynamicImports` in `.nuxt/dist/server/client.manifest.mjs`:

- compiler off: only `nuxt/dist/app/components/error-404.vue` and `error-500.vue`
- compiler on: `pages/reports.vue?vue&vize`, `pages/index.vue?vue&vize`, `…/error-404.vue?vue&vize`, `…/error-500.vue?vue&vize`

With the compiler on, `index.html` prefetches `reports.*.css` and the `reports` page chunks although the app starts on `/`.

## Expected

The same `dynamicImports` (and so the same prefetch links) as with `@vitejs/plugin-vue`: manifest keys / `src` for SFCs should be the plain file path, or the Nuxt integration should strip `?vue&vize` before Nuxt's `build:manifest` hooks run.

## Why it matters

The hook exists so that an SPA does not download every route up front. In a real app with 68 pages, the generated `index.html` gets 359 prefetch links instead of 40, and a cold visit to the sign-in page downloads 398 files instead of 81 (253 instead of 54 JS chunks). Everything still works, but the first load is several times heavier, and page-level tests that wait for the network to go idle become slow enough to time out.

## Environment

- vize 0.432.0, @vizejs/nuxt 0.432.0
- nuxt 4.5.2, vue 3.5.42
- node 26.8.1, macOS arm64
