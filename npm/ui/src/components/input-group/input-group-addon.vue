<script setup lang="ts">
defineSlots<{
  /** Renders the addon text or icon. */
  default(): unknown;
}>();
</script>

<template>
  <span data-vize-ui="input-group-addon">
    <slot />
  </span>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
