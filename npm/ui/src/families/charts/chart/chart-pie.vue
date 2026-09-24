<script setup lang="ts" generic="T">
import { computed } from "vue";
import type { ComponentPublicInstance } from "vue";

import { arcCentroid, arcPath, pieLayout } from "../chart-shape/chart-shape.ts";
import type { PieSlice } from "../chart-shape/chart-shape.ts";
import { chartContext } from "./chart-context.ts";
import { useChartNavigation } from "./chart-navigation.ts";
import { useChartSeries } from "./chart-series.ts";

const {
  data,
  value,
  label = undefined,
  innerRadius = 0,
  outerRadius = undefined,
  padAngle = 0,
  cornerRadius = 0,
  sort = "descending",
  name = undefined,
} = defineProps<{
  /** Rows to draw as slices. @default required */
  readonly data: readonly T[];

  /** Slice value of a row. @default required */
  readonly value: (datum: T, index: number) => number;

  /**
   * Accessible description of a slice. When provided, slices join keyboard navigation.
   *
   * @default undefined
   */
  readonly label?: (datum: T, index: number) => string;

  /**
   * Inner radius; greater than zero draws a donut.
   *
   * @default 0
   */
  readonly innerRadius?: number;

  /**
   * Outer radius. Defaults to half the shorter plot dimension.
   *
   * @default undefined
   */
  readonly outerRadius?: number;

  /**
   * Padding between slices in radians.
   *
   * @default 0
   */
  readonly padAngle?: number;

  /**
   * Rounded slice corners.
   *
   * @default 0
   */
  readonly cornerRadius?: number;

  /**
   * Slice drawing order.
   *
   * @default "descending"
   */
  readonly sort?: "descending" | "none" | ((left: T, right: T) => number);

  /**
   * Series name for legends, tooltips, and navigation.
   *
   * @default undefined
   */
  readonly name?: string;
}>();

defineSlots<{
  /** Optional slice label content, positioned at the slice centroid. */
  default?(props: {
    /** Layout of this slice. */
    readonly slice: PieSlice<T>;
    /** Centroid x, relative to the pie center. */
    readonly x: number;
    /** Centroid y, relative to the pie center. */
    readonly y: number;
    /** Whether this slice is active. */
    readonly active: boolean;
  }): unknown;
}>();

const context = chartContext.use();
const hidden = useChartSeries(context, () => name, "pie");
const seriesName = computed(() => name ?? `${context.id.value}-pie`);
const centerX = computed(() => context.innerWidth.value / 2);
const centerY = computed(() => context.innerHeight.value / 2);
const radius = computed(
  () => outerRadius ?? Math.min(context.innerWidth.value, context.innerHeight.value) / 2,
);
const slices = computed(() =>
  pieLayout(data, { padAngle, sort, value: (datum, index) => value(datum, index) }).map(
    (slice, index) => {
      const geometry = {
        cornerRadius,
        endAngle: slice.endAngle,
        innerRadius,
        outerRadius: radius.value,
        padAngle: slice.padAngle,
        startAngle: slice.startAngle,
      };
      const [x, y] = arcCentroid(geometry);
      return { d: arcPath(geometry), index, slice, x, y };
    },
  ),
);
const interactive = computed(() => label !== undefined && !hidden.value);
const navigation = useChartNavigation({
  context,
  count: () => slices.value.length,
  disabled: () => !interactive.value,
  point: (index) => {
    const entry = slices.value[index];
    if (entry === undefined || label === undefined) return null;
    return {
      label: label(entry.slice.data, entry.index),
      x: centerX.value + entry.x,
      y: centerY.value + entry.y,
    };
  },
  series: () => seriesName.value,
});
const groupProps = computed<{
  readonly role: "group" | "presentation";
  readonly onKeydown: (event: KeyboardEvent) => void;
}>(() => ({ role: interactive.value ? "group" : "presentation", onKeydown: navigation.onKeydown }));

interface MarkProps {
  readonly role: "img" | "presentation";
  readonly tabindex?: 0 | -1;
  readonly onFocus?: () => void;
  readonly onPointerenter?: (event: PointerEvent) => void;
  readonly onPointerleave?: (event: PointerEvent) => void;
  readonly onClick?: (event: MouseEvent) => void;
}

const markProps = computed<readonly MarkProps[]>(() =>
  slices.value.map((entry) => sliceProps(entry.index)),
);

function sliceProps(index: number): MarkProps {
  return interactive.value
    ? { ...navigation.itemProps(index), role: "img" as const }
    : { role: "presentation" as const };
}

function markRef(index: number): (element: Element | ComponentPublicInstance | null) => void {
  return (element) => navigation.registerElement(index, element);
}
</script>

<template>
  <g
    v-bind="groupProps"
    :aria-label="interactive ? name : undefined"
    :aria-hidden="interactive ? undefined : 'true'"
    :transform="`translate(${centerX},${centerY})`"
    data-vize-ui="chart-pie"
    part="pie"
    :data-series="name"
    :data-hidden="hidden ? 'true' : undefined"
  >
    <template v-if="!hidden">
      <g v-for="entry in slices" :key="entry.index" data-vize-ui="chart-slice-group">
        <path
          :ref="markRef(entry.index)"
          v-bind="markProps[entry.index]"
          :aria-label="label === undefined ? undefined : label(entry.slice.data, entry.index)"
          :d="entry.d"
          data-vize-ui="chart-slice"
          part="slice"
          :data-index="entry.index"
          :data-active="navigation.activeIndex.value === entry.index ? 'true' : undefined"
        />
        <slot
          :slice="entry.slice"
          :x="entry.x"
          :y="entry.y"
          :active="navigation.activeIndex.value === entry.index"
        />
      </g>
    </template>
  </g>
</template>

<style scoped>
/* Headless by design. Fill and focus styling remain consumer-owned. */
</style>
