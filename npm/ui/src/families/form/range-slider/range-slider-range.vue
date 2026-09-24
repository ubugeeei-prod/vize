<script setup lang="ts">
import { rangeSliderContext } from "./range-slider-context.ts";

defineSlots<{
  /** Optional range contents. */
  default?(props: Record<string, never>): unknown;
}>();

const context = rangeSliderContext.use();
</script>

<template>
  <span
    part="range"
    data-vize-ui="range-slider-range"
    :data-orientation="context.orientation.value"
    :data-disabled="context.disabled.value ? 'true' : undefined"
  >
    <slot />
  </span>
</template>

<style scoped>
/* Headless by design. Position with --vize-range-slider-range-start/end from the root. */
</style>
