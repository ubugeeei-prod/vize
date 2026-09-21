<script setup lang="ts">
function inlineFormat(value: boolean, count: number): string {
  return String(value).repeat(count)
}
</script>

<template><main /></template>

<art>
  <variant name="Inline">
    <p>{{ inlineFormat(true, 2) }}</p>
  </variant>
</art>
