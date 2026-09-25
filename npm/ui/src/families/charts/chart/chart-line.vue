<script setup lang="ts" generic="T">
import { computed } from "vue";

import { linePath } from "../chart-shape/chart-shape.ts";
import { chartContext } from "./chart-context.ts";
import { useChartSeries } from "./chart-series.ts";
import type { ChartCurve } from "./chart-types.ts";

const {
  data,
  x,
  y,
  defined = undefined,
  curve = "linear",
  name = undefined,
} = defineProps<{
  /** Rows to draw, in drawing order. @default required */
  readonly data: readonly T[];

  /** Plot-area x coordinate of a row, usually `(row) => xScale(row.date)`. @default required */
  readonly x: (datum: T, index: number) => number;

  /** Plot-area y coordinate of a row. @default required */
  readonly y: (datum: T, index: number) => number;

  /**
   * Rows for which this returns `false` break the line into segments.
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
   * Series name for legends and data tables. Named series can be hidden by the legend.
   *
   * @default undefined
   */
  readonly name?: string;
}>();

const context = chartContext.use();
const hidden = useChartSeries(context, () => name, "line");
const d = computed(
  () =>
    linePath(data, {
      curve,
      x: (datum, index) => x(datum, index),
      y: (datum, index) => y(datum, index),
      ...(defined === undefined
        ? {}
        : { defined: (datum: T, index: number) => defined(datum, index) }),
    }) ?? undefined,
);
</script>

<template>
  <path
    aria-hidden="true"
    :d="hidden ? undefined : d"
    fill="none"
    data-vize-ui="chart-line"
    part="line"
    :data-series="name"
    :data-hidden="hidden ? 'true' : undefined"
  />
</template>

<style scoped>
/* Headless by design. Stroke styling remains consumer-owned. */
</style>
