<script setup lang="ts">
import { computed } from "vue";

import { tourContext } from "./tour-context.ts";
import type { TourProgressSlotState } from "./tour-types.ts";

defineSlots<{
  /** Progress text. Defaults to `current / total`. */
  default(props: TourProgressSlotState): unknown;
}>();

const context = tourContext.use();
const index = computed(() => context.index.value);
const current = computed(() => context.index.value + 1);
const total = computed(() => context.total.value);
const fraction = computed(() =>
  context.total.value === 0 || context.index.value < 0
    ? 0
    : (context.index.value + 1) / context.total.value,
);
const slotState = computed<TourProgressSlotState>(() => ({
  current: current.value,
  fraction: fraction.value,
  index: index.value,
  total: total.value,
}));
</script>

<template>
  <span
    data-vize-ui="tour-progress"
    part="progress"
    :data-state="context.state.value"
    :data-current="current"
    :data-total="total"
  >
    <slot v-bind="slotState">{{ current }} / {{ total }}</slot>
  </span>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
