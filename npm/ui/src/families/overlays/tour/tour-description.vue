<script setup lang="ts">
import { tourContext } from "./tour-context.ts";
import type { TourSlotState } from "./tour-types.ts";

defineSlots<{
  /** Description text. Receives the current step and progress. */
  default(props: TourSlotState): unknown;
}>();

const context = tourContext.use();
</script>

<template>
  <div
    :id="context.descriptionId.value"
    data-vize-ui="tour-description"
    part="description"
    :data-state="context.state.value"
    :data-step="context.step.value?.value"
  >
    <slot v-bind="context.slotState.value" />
  </div>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
