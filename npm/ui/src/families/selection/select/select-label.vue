<script setup lang="ts">
import { selectContext, selectGroupContext } from "./select-context.ts";

defineSlots<{
  /** Visible group heading text. */
  default(): unknown;
}>();

const context = selectContext.use();
const group = selectGroupContext.use();
</script>

<template>
  <div
    :id="group.labelId.value"
    role="presentation"
    :data-vize-ui="`${context.partPrefix}-label`"
    part="label"
  >
    <slot />
  </div>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
