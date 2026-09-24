<script setup lang="ts" generic="T">
import { computed } from "vue";
import type { ComponentPublicInstance } from "vue";

import { chartContext } from "./chart-context.ts";
import { useChartNavigation } from "./chart-navigation.ts";
import { useChartSeries } from "./chart-series.ts";

const {
  data,
  x,
  y,
  label,
  name,
  radius = 4,
  capturePointer = true,
} = defineProps<{
  /** Rows to mark, in keyboard navigation order. @default required */
  readonly data: readonly T[];

  /** Plot-area x coordinate of a row. @default required */
  readonly x: (datum: T, index: number) => number;

  /** Plot-area y coordinate of a row. @default required */
  readonly y: (datum: T, index: number) => number;

  /** Accessible description of a row, announced when it receives focus. @default required */
  readonly label: (datum: T, index: number) => string;

  /** Series name, used as the group label and for legends and tooltips. @default required */
  readonly name: string;

  /**
   * Point radius in CSS pixels.
   *
   * @default 4
   */
  readonly radius?: number;

  /**
   * Activate the nearest point while the pointer moves anywhere over the plot area.
   *
   * @default true
   */
  readonly capturePointer?: boolean;
}>();

const emit = defineEmits<{
  /** Fired when a point is activated with Enter or Space, or clicked. */
  select: [datum: T, index: number, nativeEvent: Event];
}>();

defineSlots<{
  /** Optional decoration drawn inside each point group. Receives the datum and state. */
  default?(props: {
    /** Row of this point. */
    readonly datum: T;
    /** Index of the row. */
    readonly index: number;
    /** Whether this point is active. */
    readonly active: boolean;
    /** Plot-area x coordinate. */
    readonly x: number;
    /** Plot-area y coordinate. */
    readonly y: number;
  }): unknown;
}>();

const context = chartContext.use();
const hidden = useChartSeries(context, () => name, "points");
const points = computed(() =>
  data.map((datum, index) => ({ datum, index, x: x(datum, index), y: y(datum, index) })),
);
const navigation = useChartNavigation({
  context,
  count: () => points.value.length,
  disabled: () => hidden.value,
  point: (index) => {
    const point = points.value[index];
    return point === undefined
      ? null
      : { label: label(point.datum, index), x: point.x, y: point.y };
  },
  series: () => name,
});

function select(index: number, event: Event): void {
  const point = points.value[index];
  if (point !== undefined) emit("select", point.datum, index, event);
}

function onKeydown(event: KeyboardEvent): void {
  if (event.key === "Enter" || event.key === " ") {
    const index = navigation.activeIndex.value;
    if (index === null) return;
    event.preventDefault();
    select(index, event);
    return;
  }
  navigation.onKeydown(event);
}

const groupProps = computed<{
  readonly role: "group";
  readonly onKeydown: (event: KeyboardEvent) => void;
}>(() => ({ role: "group", onKeydown }));

const captureProps = computed<{
  readonly onPointermove: (event: PointerEvent) => void;
  readonly onPointerleave: () => void;
}>(() => ({
  onPointermove: (event: PointerEvent) => {
    if (event.pointerType !== "touch") navigation.activateNearest(event);
  },
  onPointerleave: () => navigation.deactivate(),
}));

interface MarkProps {
  readonly role: "img" | "presentation";
  readonly tabindex?: 0 | -1;
  readonly onFocus?: () => void;
  readonly onPointerenter?: (event: PointerEvent) => void;
  readonly onPointerleave?: (event: PointerEvent) => void;
  readonly onClick?: (event: MouseEvent) => void;
}

const markProps = computed<readonly MarkProps[]>(() =>
  points.value.map((point) => pointProps(point.index)),
);

function pointProps(index: number): MarkProps {
  return {
    ...navigation.itemProps(index),
    role: "img" as const,
    onClick: (event: MouseEvent) => select(index, event),
  };
}

function markRef(index: number): (element: Element | ComponentPublicInstance | null) => void {
  return (element) => navigation.registerElement(index, element);
}
</script>

<template>
  <g
    v-bind="groupProps"
    :aria-label="name"
    data-vize-ui="chart-points"
    part="points"
    :data-series="name"
    :data-hidden="hidden ? 'true' : undefined"
  >
    <rect
      v-if="capturePointer && !hidden"
      v-bind="captureProps"
      aria-hidden="true"
      :width="context.innerWidth.value"
      :height="context.innerHeight.value"
      fill="transparent"
      data-vize-ui="chart-points-capture"
    />
    <template v-if="!hidden">
      <g
        v-for="point in points"
        :key="point.index"
        :ref="markRef(point.index)"
        v-bind="markProps[point.index]"
        :aria-label="label(point.datum, point.index)"
        :transform="`translate(${point.x},${point.y})`"
        data-vize-ui="chart-point"
        part="point"
        :data-index="point.index"
        :data-active="navigation.activeIndex.value === point.index ? 'true' : undefined"
      >
        <circle :r="radius" data-vize-ui="chart-point-mark" />
        <slot
          :datum="point.datum"
          :index="point.index"
          :active="navigation.activeIndex.value === point.index"
          :x="point.x"
          :y="point.y"
        />
      </g>
    </template>
  </g>
</template>

<style scoped>
/* Headless by design. Mark and focus styling remain consumer-owned. */
</style>
