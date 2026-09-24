<script setup lang="ts">
import { schedulerContext } from "./scheduler-context.ts";

defineSlots<{
  /** Heading content. Receives the localized period label. */
  default(props: { readonly heading: string }): unknown;
}>();

const context = schedulerContext.use();
</script>

<template>
  <h2
    :id="context.headingId.value"
    aria-live="polite"
    aria-atomic="true"
    data-vize-ui="scheduler-heading"
    part="heading"
  >
    <slot :heading="context.slotState.value.heading">{{ context.slotState.value.heading }}</slot>
  </h2>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
