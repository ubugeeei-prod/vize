<script setup lang="ts">
import { rangeSliderContext } from "./range-slider-context.ts";

defineSlots<{
  /** Track contents, typically RangeSliderRange and the thumbs. */
  default?(props: Record<string, never>): unknown;
}>();

const context = rangeSliderContext.use();

function onPointerdown(event: PointerEvent): void {
  if (!(event.currentTarget instanceof HTMLElement)) return;
  context.startDrag(event, event.currentTarget);
}
</script>

<template>
  <span
    part="track"
    data-vize-ui="range-slider-track"
    :data-orientation="context.orientation.value"
    :data-disabled="context.disabled.value ? 'true' : undefined"
    @pointerdown="onPointerdown"
  >
    <slot />
  </span>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
