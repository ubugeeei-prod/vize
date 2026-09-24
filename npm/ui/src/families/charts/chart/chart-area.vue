<script setup lang="ts" generic="T">
import { computed } from "vue";

import { areaPath } from "../chart-shape/chart-shape.ts";
import { chartContext } from "./chart-context.ts";
import { useChartSeries } from "./chart-series.ts";
import type { ChartCurve } from "./chart-types.ts";

const {
  data,
  x,
  y,
  y0 = undefined,
  defined = undefined,
  curve = "linear",
  name = undefined,
} = defineProps<{
  /** Rows to draw, in drawing order. @default required */
  readonly data: readonly T[];

  /** Plot-area x coordinate of a row. @default required */
  readonly x: (datum: T, index: number) => number;

  /** Plot-area y coordinate of the area's top edge. @default required */
  readonly y: (datum: T, index: number) => number;

  /**
   * Plot-area y coordinate of the baseline. Defaults to the bottom of the plot area.
   *
   * @default undefined
   */
  readonly y0?: number | ((datum: T, index: number) => number);

  /**
   * Rows for which this returns `false` break the area into segments.
   *
   * @default undefined
   */
  readonly defined?: (datum: T, index: number) => boolean;

  /**
   * Curve interpolation.
   *
   * @default "linear"
   */
  readonly curve?: ChartCurve;

  /**
   * Series name for legends and data tables.
   *
   * @default undefined
   */
  readonly name?: string;
}>();

const context = chartContext.use();
const hidden = useChartSeries(context, () => name, "area");
const d = computed(() => {
  const baseline = y0 ?? context.innerHeight.value;
  return (
    areaPath(data, {
      curve,
      x: (datum, index) => x(datum, index),
      y0: typeof baseline === "number" ? baseline : (datum, index) => baseline(datum, index),
      y1: (datum, index) => y(datum, index),
      ...(defined === undefined
        ? {}
        : { defined: (datum: T, index: number) => defined(datum, index) }),
    }) ?? undefined
  );
});
</script>

<template>
  <path
    aria-hidden="true"
    :d="hidden ? undefined : d"
    data-vize-ui="chart-area"
    part="area"
    :data-series="name"
    :data-hidden="hidden ? 'true' : undefined"
  />
</template>

<style scoped>
/* Headless by design. Fill styling remains consumer-owned. */
</style>
