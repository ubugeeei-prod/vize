<script setup lang="ts">
// Negative control. Nothing in the generated `.nuxt` graph declares `$missing`,
// so both `vue-tsc` and `vize check` must report it — once per authored use —
// rather than resolving it to `any`. `notDeclaredAnywhere` is the value-side
// control: a plain undefined name stays TS2304 and must not be absorbed into
// the instance lookup.
const value = notDeclaredAnywhere()
</script>

<template>
  <p>{{ $missing('a') }}</p>
  <p>{{ $missing('b') }}</p>
  <p :title="$missing('c')">{{ value }}</p>
</template>
