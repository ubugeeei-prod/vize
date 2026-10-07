## Summary

With `--cross-file`, route typing builds the project's route table from every `createRouter({ routes })` it finds in the linted files. A throwaway router created in a component test (or a component-gallery preview) then becomes "the" route table, and every named navigation in the app is reported by `ecosystem/vue-router-unknown-route`.

In a Nuxt app the real routes come from `pages/` (file-based routing) and are never registered with `createRouter` in user code, so the only routers vize can see are test/preview routers. The result is an error on every `router.push({ name })`, even when the name matches an existing page.

## Reproduction

```
nuxt.config.ts            export default defineNuxtConfig({ srcDir: "src" })
src/pages/index.vue
src/pages/users/[id]/index.vue      -> Nuxt route name "users-id"
src/components/user-link.vue
src/components/user-link.test.ts
```

`src/components/user-link.vue`:

```vue
<script setup lang="ts">
import { useRouter } from "vue-router";

const router = useRouter();

function open(id: number): void {
  router.push({ name: "users-id", params: { id: String(id) } });
}
</script>

<template>
  <button type="button" @click="open(1)">open</button>
</template>
```

`src/components/user-link.test.ts`:

```ts
import { createMemoryHistory, createRouter } from "vue-router";

// A throwaway router for a component test. It is not the app's route table.
export const router = createRouter({
  history: createMemoryHistory(),
  routes: [{ path: "/", component: { template: "<div />" } }],
});
```

```sh
vize lint --cross-file "src/**/*.vue" "src/**/*.ts"
```

## Actual (0.432.0)

```text
× [vize:ecosystem/vue-router-unknown-route] unknown route name `users-id`
  ╭─[src/components/user-link.vue:7:23]
7 │   router.push({ name: "users-id", params: { id: String(id) } });
  ·                       ──────────
  help: known routes: none (router at src/components/user-link.test.ts:4:23)
```

Without the test file nothing is reported, so the diagnostic depends only on whether some test happens to create a router.

## Expected

- In a Nuxt app, resolve route names from the file-based pages (`ecosystem/vue-router-route-param` already reads page file names, see #7816), or skip the check when the app's routes are not defined through `createRouter`.
- A router created in a test file (or with `createMemoryHistory()` for a test/preview) should not be treated as the application's route table for other files.

## Environment

- vize / @vizejs/native 0.432.0
- vue 3.5.38, vue-router 4.5
- macOS arm64, Node 26
