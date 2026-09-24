<script setup lang="ts" generic="Tick">
import { computed } from "vue";

import { chartContext } from "./chart-context.ts";
import { defaultTickFormatter } from "./chart-format.ts";
import type { ChartAxisOrientation, ChartAxisScale, ChartAxisTick } from "./chart-types.ts";

const {
  scale,
  orientation = "bottom",
  tickCount = 5,
  tickValues = undefined,
  format = undefined,
  locale = "en-US",
  tickSize = 6,
  tickPadding = 3,
  label = undefined,
} = defineProps<{
  /** Scale positioning the ticks, such as a linear, time, or band scale. @default required */
  readonly scale: ChartAxisScale<Tick>;

  /**
   * Side of the plot area the axis is drawn on.
   *
   * @default "bottom"
   */
  readonly orientation?: ChartAxisOrientation;

  /**
   * Approximate number of ticks requested from continuous scales.
   *
   * @default 5
   */
  readonly tickCount?: number;

  /**
   * Explicit tick values that replace `scale.ticks()`.
   *
   * @default undefined
   */
  readonly tickValues?: readonly Tick[];

  /**
   * Tick label formatter. Defaults to the scale-aware `Intl` formatter.
   *
   * @default undefined
   */
  readonly format?: (tick: Tick, index: number) => string;

  /**
   * Locale of the default formatter. Set it explicitly for server rendering.
   *
   * @default "en-US"
   */
  readonly locale?: string;

  /**
   * Length of tick marks in CSS pixels.
   *
   * @default 6
   */
  readonly tickSize?: number;

  /**
   * Gap between a tick mark and its label.
   *
   * @default 3
   */
  readonly tickPadding?: number;

  /**
   * Axis title rendered beside the ticks.
   *
   * @default undefined
   */
  readonly label?: string;
}>();

defineSlots<{
  /** Custom tick label content. Receives the tick value, label, and offset. */
  tick?(props: ChartAxisTick<Tick>): unknown;
}>();

const context = chartContext.use();
const vertical = computed(() => orientation === "left" || orientation === "right");
const direction = computed(() => (orientation === "top" || orientation === "left" ? -1 : 1));
const values = computed<readonly Tick[]>(() => tickValues ?? scale.ticks(tickCount));
const center = computed(() => (scale.bandwidth ?? 0) / 2);
const formatter = computed(() => format ?? defaultTickFormatter(scale, values.value, locale));
const ticks = computed<ChartAxisTick<Tick>[]>(() =>
  values.value.map((value, index) => ({
    index,
    label: formatter.value(value, index),
    offset: scale(value) + center.value,
    value,
  })),
);
const transform = computed(() => {
  if (orientation === "bottom") return `translate(0,${context.innerHeight.value})`;
  if (orientation === "right") return `translate(${context.innerWidth.value},0)`;
  return undefined;
});
const rangeStart = computed(() =>
  scale.range.reduce((low, value) => Math.min(low, value), Infinity),
);
const rangeEnd = computed(() =>
  scale.range.reduce((high, value) => Math.max(high, value), -Infinity),
);
const domainPath = computed(() =>
  vertical.value
    ? `M0,${rangeStart.value}V${rangeEnd.value}`
    : `M${rangeStart.value},0H${rangeEnd.value}`,
);
const labelOffset = computed(() => direction.value * (tickSize + tickPadding));
const textAnchor = computed(() => {
  if (orientation === "left") return "end";
  if (orientation === "right") return "start";
  return "middle";
});
const dominantBaseline = computed(() => {
  if (orientation === "bottom") return "hanging";
  if (orientation === "top") return "auto";
  return "middle";
});
</script>

<template>
  <g
    aria-hidden="true"
    :transform="transform"
    data-vize-ui="chart-axis"
    part="axis"
    :data-orientation="orientation"
  >
    <path :d="domainPath" data-vize-ui="chart-axis-domain" part="domain" fill="none" />
    <g
      v-for="tick in ticks"
      :key="tick.index"
      :transform="vertical ? `translate(0,${tick.offset})` : `translate(${tick.offset},0)`"
      data-vize-ui="chart-axis-tick"
      part="tick"
    >
      <line
        :x2="vertical ? direction * tickSize : 0"
        :y2="vertical ? 0 : direction * tickSize"
        data-vize-ui="chart-axis-tick-line"
      />
      <text
        :x="vertical ? labelOffset : 0"
        :y="vertical ? 0 : labelOffset"
        :text-anchor="textAnchor"
        :dominant-baseline="dominantBaseline"
        data-vize-ui="chart-axis-tick-label"
      >
        <slot name="tick" v-bind="tick">{{ tick.label }}</slot>
      </text>
    </g>
    <text
      v-if="label !== undefined"
      data-vize-ui="chart-axis-label"
      part="label"
      :text-anchor="vertical ? 'middle' : 'end'"
      :transform="vertical ? `rotate(-90)` : undefined"
      :x="vertical ? -(rangeStart + rangeEnd) / 2 : rangeEnd"
      :y="
        vertical
          ? direction * (tickSize + tickPadding) * 5
          : direction * (tickSize + tickPadding) * 4
      "
    >
      {{ label }}
    </text>
  </g>
</template>

<style scoped>
/* Headless by design. Stroke, fill, and typography remain consumer-owned. */
</style>
