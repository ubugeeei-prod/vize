<script setup lang="ts">
import { selectContext } from "./select-context.ts";

defineSlots<{
  /** Optional decorative separator content. */
  default?(): unknown;
}>();

const context = selectContext.use();
</script>

<template>
  <div aria-hidden="true" :data-vize-ui="`${context.partPrefix}-separator`" part="separator">
    <slot />
  </div>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
