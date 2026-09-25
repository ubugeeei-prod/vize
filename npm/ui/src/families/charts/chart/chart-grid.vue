<script setup lang="ts" generic="Tick">
import { computed } from "vue";

import { chartContext } from "./chart-context.ts";
import type { ChartAxisScale } from "./chart-types.ts";

const {
  scale,
  direction = "horizontal",
  tickCount = 5,
  tickValues = undefined,
} = defineProps<{
  /** Scale positioning the grid lines. @default required */
  readonly scale: ChartAxisScale<Tick>;

  /**
   * `"horizontal"` draws lines across the plot for a y scale; `"vertical"` for an x scale.
   *
   * @default "horizontal"
   */
  readonly direction?: "horizontal" | "vertical";

  /**
   * Approximate number of lines requested from continuous scales.
   *
   * @default 5
   */
  readonly tickCount?: number;

  /**
   * Explicit values that replace `scale.ticks()`.
   *
   * @default undefined
   */
  readonly tickValues?: readonly Tick[];
}>();

const context = chartContext.use();
const center = computed(() => (scale.bandwidth ?? 0) / 2);
const lines = computed(() =>
  (tickValues ?? scale.ticks(tickCount)).map((value, index) => ({
    index,
    offset: scale(value) + center.value,
  })),
);
</script>

<template>
  <g aria-hidden="true" data-vize-ui="chart-grid" part="grid" :data-direction="direction">
    <line
      v-for="line in lines"
      :key="line.index"
      :x1="direction === 'horizontal' ? 0 : line.offset"
      :x2="direction === 'horizontal' ? context.innerWidth.value : line.offset"
      :y1="direction === 'horizontal' ? line.offset : 0"
      :y2="direction === 'horizontal' ? line.offset : context.innerHeight.value"
      data-vize-ui="chart-grid-line"
      part="line"
    />
  </g>
</template>

<style scoped>
/* Headless by design. Stroke styling remains consumer-owned. */
</style>
