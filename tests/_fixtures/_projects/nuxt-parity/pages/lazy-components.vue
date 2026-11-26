<script setup lang="ts">
// INTENTIONAL type errors. `<LazyTheGreeting>` is typed through the
// `LazyComponent` alias local to `.nuxt/types/components.d.ts`, so its `name`
// prop must stay a `string` (TS2322). The manifest re-exports no `UnwrapRef`,
// so the name is not in scope (TS2304), as under vue-tsc.
const wrapped: UnwrapRef<number> = 1;
</script>

<template>
  <LazyTheGreeting :name="5" />
  <LazyTheGreeting name="lazy" :hydrate-never="true" />
  <p>{{ wrapped }}</p>
</template>
