<script setup lang="ts" generic="T, Category">
import { computed } from "vue";
import type { ComponentPublicInstance } from "vue";

import { barRects, roundedRectPath } from "../chart-shape/chart-shape.ts";
import type { BarCategoryScale, BarOrientation } from "../chart-shape/chart-shape.ts";
import { chartContext } from "./chart-context.ts";
import { useChartNavigation } from "./chart-navigation.ts";
import { useChartSeries } from "./chart-series.ts";

const {
  data,
  category,
  value,
  categoryScale,
  valueScale,
  orientation = "vertical",
  baseline = 0,
  radius = 0,
  label = undefined,
  name = undefined,
} = defineProps<{
  /** Rows to draw as bars. @default required */
  readonly data: readonly T[];

  /** Category of a row. @default required */
  readonly category: (datum: T, index: number) => Category;

  /** Value of a row. @default required */
  readonly value: (datum: T, index: number) => number;

  /** Band scale positioning categories. @default required */
  readonly categoryScale: BarCategoryScale<Category>;

  /** Continuous scale positioning values. @default required */
  readonly valueScale: (value: number) => number;

  /**
   * Direction bars grow in.
   *
   * @default "vertical"
   */
  readonly orientation?: BarOrientation;

  /**
   * Value bars start from, or a per-row baseline for stacked bars.
   *
   * @default 0
   */
  readonly baseline?: number | ((datum: T, index: number) => number);

  /**
   * Corner radius applied to every bar.
   *
   * @default 0
   */
  readonly radius?: number;

  /**
   * Accessible description of a bar. When provided, bars join keyboard navigation.
   *
   * @default undefined
   */
  readonly label?: (datum: T, index: number) => string;

  /**
   * Series name for legends, tooltips, and navigation.
   *
   * @default undefined
   */
  readonly name?: string;
}>();

const context = chartContext.use();
const hidden = useChartSeries(context, () => name, "bars");
const seriesName = computed(() => name ?? `${context.id.value}-bars`);
const rects = computed(() =>
  barRects(data, {
    baseline: typeof baseline === "number" ? baseline : (datum, index) => baseline(datum, index),
    category: (datum, index) => category(datum, index),
    categoryScale,
    orientation,
    value: (datum, index) => value(datum, index),
    valueScale,
  }),
);
const interactive = computed(() => label !== undefined && !hidden.value);
const navigation = useChartNavigation({
  context,
  count: () => rects.value.length,
  disabled: () => !interactive.value,
  point: (index) => {
    const rect = rects.value[index];
    if (rect === undefined || label === undefined) return null;
    const vertical = orientation === "vertical";
    return {
      label: label(rect.data, index),
      x: vertical ? rect.x + rect.width / 2 : rect.x + rect.width,
      y: vertical ? rect.y : rect.y + rect.height / 2,
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
  rects.value.map((rect) => barProps(rect.index)),
);

function barProps(index: number): MarkProps {
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
    data-vize-ui="chart-bars"
    part="bars"
    :data-series="name"
    :data-orientation="orientation"
    :data-hidden="hidden ? 'true' : undefined"
  >
    <template v-if="!hidden">
      <path
        v-for="rect in rects"
        :key="rect.index"
        :ref="markRef(rect.index)"
        v-bind="markProps[rect.index]"
        :aria-label="label === undefined ? undefined : label(rect.data, rect.index)"
        :d="roundedRectPath(rect, radius)"
        data-vize-ui="chart-bar"
        part="bar"
        :data-index="rect.index"
        :data-negative="rect.value < 0 ? 'true' : undefined"
        :data-active="navigation.activeIndex.value === rect.index ? 'true' : undefined"
      />
    </template>
  </g>
</template>

<style scoped>
/* Headless by design. Fill and focus styling remain consumer-owned. */
</style>
