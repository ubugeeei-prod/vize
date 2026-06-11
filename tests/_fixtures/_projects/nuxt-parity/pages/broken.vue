<script setup lang="ts">
// INTENTIONAL type error: `useCounter` is auto-imported and strongly typed to
// accept a `number`. Passing a string must be caught (TS2345) — proving the
// auto-import resolves to real types and is not silently `any`. If `vize check`
// ever stops reporting this, the Nuxt `any`-stub regression is back.
const counter = useCounter("not a number");
counter.increment(1);
</script>

<template>
  <p>{{ counter.count }}</p>
</template>
