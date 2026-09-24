<script setup lang="ts">
import { tourContext } from "./tour-context.ts";
import type { TourSlotState } from "./tour-types.ts";

const { as = "h2" } = defineProps<{
  /**
   * Heading element rendered for the step title.
   *
   * @default "h2"
   */
  readonly as?: "h1" | "h2" | "h3" | "h4" | "h5" | "h6" | "p";
}>();

defineSlots<{
  /** Title text. Receives the current step and progress. */
  default(props: TourSlotState): unknown;
}>();

const context = tourContext.use();
</script>

<template>
  <component
    :is="as"
    :id="context.titleId.value"
    data-vize-ui="tour-title"
    part="title"
    :data-state="context.state.value"
    :data-step="context.step.value?.value"
  >
    <slot v-bind="context.slotState.value" />
  </component>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
