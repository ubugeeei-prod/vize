<script setup lang="ts">
import { fieldsetContext } from "./fieldset-context.ts";

defineSlots<{
  /** Legend text that names the group; render it as the first child of the Fieldset. */
  default(props: Record<string, never>): unknown;
}>();

const context = fieldsetContext.use();
</script>

<template>
  <legend
    part="legend"
    data-vize-ui="fieldset-legend"
    :data-invalid="context.invalid.value ? 'true' : 'false'"
  >
    <slot />
  </legend>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
