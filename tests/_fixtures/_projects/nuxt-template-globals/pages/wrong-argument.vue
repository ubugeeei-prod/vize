<script setup lang="ts">
// INTENTIONAL type errors. The auto-imports and template globals resolve to
// their real declared signatures, so a wrong-typed call is a real TS2345 in
// both the script and the template. If any of these stops being reported, an
// `any` stand-in has come back in place of the generated declaration.
const localePath = useLocalePath()
const wrongAlias = localePath(1)
</script>

<template>
  <p>{{ wrongAlias }} {{ $t(2) }} {{ $shout(3) }}</p>
</template>
