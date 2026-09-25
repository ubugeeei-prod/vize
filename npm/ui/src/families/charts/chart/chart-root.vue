<script setup lang="ts">
import {
  computed,
  onMounted,
  onScopeDispose,
  shallowReactive,
  shallowRef,
  useTemplateRef,
} from "vue";
import type { ComputedRef } from "vue";

import { announcerContext } from "../../accessibility/announcer/announcer.ts";
import { useControllableState } from "../../foundations/controllable-state/controllable-state.ts";
import {
  deriveDeterministicId,
  useDeterministicId,
} from "../../foundations/id/deterministic-id.ts";
import { chartContext } from "./chart-context.ts";
import type { ChartContextValue } from "./chart-context.ts";
import type {
  ChartActivePoint,
  ChartActiveReason,
  ChartMargin,
  ChartRootExpose,
  ChartSeriesInfo,
  ChartSeriesKind,
  ChartSlotState,
} from "./chart-types.ts";

const {
  id = undefined,
  width = undefined,
  height = 360,
  defaultWidth = 640,
  margin = undefined,
  title,
  description = undefined,
  hiddenSeries = undefined,
  defaultHiddenSeries = undefined,
} = defineProps<{
  /**
   * Consumer-owned chart id. `null` and `undefined` select a deterministic fallback.
   *
   * @default undefined
   */
  readonly id?: string | null;

  /**
   * Fixed width in CSS pixels. `undefined` makes the chart responsive: it
   * renders `defaultWidth` on the server and then follows its container.
   *
   * @default undefined
   */
  readonly width?: number;

  /**
   * Height in CSS pixels.
   *
   * @default 360
   */
  readonly height?: number;

  /**
   * Width used for server rendering and before the container is measured.
   *
   * @default 640
   */
  readonly defaultWidth?: number;

  /**
   * Space around the plot area for axes. Missing sides keep their defaults.
   *
   * @default { top: 16, right: 16, bottom: 32, left: 40 }
   */
  readonly margin?: Partial<ChartMargin>;

  /** Accessible chart title, rendered as the SVG `<title>`. @default required */
  readonly title: string;

  /**
   * Longer accessible summary, rendered as the SVG `<desc>`.
   *
   * @default undefined
   */
  readonly description?: string;

  /**
   * Controlled hidden series names (`v-model:hiddenSeries`).
   *
   * @default undefined
   */
  readonly hiddenSeries?: readonly string[];

  /**
   * Initially hidden series names for uncontrolled use.
   *
   * @default []
   */
  readonly defaultHiddenSeries?: readonly string[];
}>();

const emit = defineEmits<{
  /** Fired with the next hidden series names when the legend toggles a series. */
  "update:hiddenSeries": [names: readonly string[]];

  /** Fired when keyboard focus or pointer hover changes the active data point. */
  activeChange: [point: ChartActivePoint | null, reason: ChartActiveReason];
}>();

defineSlots<{
  /** SVG chart parts (axes, grids, series, points), rendered inside the plot area. */
  default(props: ChartSlotState): unknown;

  /** HTML layered over the chart, such as ChartTooltip. */
  overlay?(props: ChartSlotState): unknown;

  /** HTML after the chart, such as ChartLegend and ChartDataTable. */
  after?(props: ChartSlotState): unknown;
}>();

const noNames: readonly string[] = Object.freeze([]);
const element = useTemplateRef<HTMLElement>("element");
const svgElement = useTemplateRef<SVGSVGElement>("svg");
const baseId = useDeterministicId({ id: () => id, hint: "chart" });
const titleId = computed(() => deriveDeterministicId(baseId.value, "title"));
const descriptionId = computed(() => deriveDeterministicId(baseId.value, "description"));
const measuredWidth = shallowRef<number | null>(null);
const resolvedWidth = computed(() => width ?? measuredWidth.value ?? defaultWidth);
const resolvedHeight = computed(() => height);
const resolvedMargin = computed<ChartMargin>(() => ({
  top: margin?.top ?? 16,
  right: margin?.right ?? 16,
  bottom: margin?.bottom ?? 32,
  left: margin?.left ?? 40,
}));
const innerWidth = computed(() =>
  Math.max(0, resolvedWidth.value - resolvedMargin.value.left - resolvedMargin.value.right),
);
const innerHeight = computed(() =>
  Math.max(0, resolvedHeight.value - resolvedMargin.value.top - resolvedMargin.value.bottom),
);
const registered = shallowReactive(new Map<string, ChartSeriesKind>());
const hiddenState = useControllableState<readonly string[]>({
  value: () => hiddenSeries,
  defaultValue: () => defaultHiddenSeries ?? noNames,
});
const hiddenSet = computed(() => new Set(hiddenState.value.value));
const series = computed<readonly ChartSeriesInfo[]>(() => {
  const entries: ChartSeriesInfo[] = [];
  registered.forEach((kind, name) => {
    entries.push({ name, kind, hidden: hiddenSet.value.has(name) });
  });
  return entries;
});
const active = shallowRef<ChartActivePoint | null>(null);
const announcement = shallowRef("");
const externalAnnouncer = announcerContext.useOptional();
const plotTransform = computed(
  () => `translate(${resolvedMargin.value.left},${resolvedMargin.value.top})`,
);
const slotState = computed<ChartSlotState>(() => ({
  active: active.value,
  height: resolvedHeight.value,
  innerHeight: innerHeight.value,
  innerWidth: innerWidth.value,
  margin: resolvedMargin.value,
  series: series.value,
  width: resolvedWidth.value,
}));

// Visually hidden without packaged CSS: the region must stay in the accessibility tree.
const liveRegionStyle = Object.freeze({
  border: "0",
  clipPath: "inset(50%)",
  height: "1px",
  margin: "-1px",
  overflow: "hidden",
  padding: "0",
  position: "absolute",
  whiteSpace: "nowrap",
  width: "1px",
});

function toggleSeries(name: string, hidden?: boolean): boolean {
  const next = hidden ?? !hiddenSet.value.has(name);
  const current = hiddenState.value.value;
  const updated = next
    ? [...new Set([...current, name])]
    : current.filter((entry) => entry !== name);
  const changed = hiddenState.set(updated);
  if (changed) emit("update:hiddenSeries", updated);
  if (changed && next && active.value?.series === name) setActive(null, "programmatic");
  return changed;
}

function readActive(): ChartActivePoint | null {
  return active.value;
}

function readSvg(): SVGSVGElement | null {
  return svgElement.value;
}

function readElement(): HTMLElement | null {
  return element.value;
}

function setActive(point: ChartActivePoint | null, reason: ChartActiveReason): void {
  const previous = readActive();
  if (
    previous === point ||
    (previous !== null &&
      point !== null &&
      previous.series === point.series &&
      previous.index === point.index &&
      previous.x === point.x &&
      previous.y === point.y)
  ) {
    return;
  }
  active.value = point;
  emit("activeChange", point, reason);
}

function announce(text: string): void {
  if (externalAnnouncer !== undefined) {
    externalAnnouncer.announce(text);
    return;
  }
  announcement.value = text;
}

function plotPoint(event: MouseEvent): { readonly x: number; readonly y: number } | null {
  const svg = readSvg();
  if (svg === null) return null;
  const rect = svg.getBoundingClientRect();
  const scaleX = rect.width > 0 ? resolvedWidth.value / rect.width : 1;
  const scaleY = rect.height > 0 ? resolvedHeight.value / rect.height : 1;
  return {
    x: (event.clientX - rect.left) * scaleX - resolvedMargin.value.left,
    y: (event.clientY - rect.top) * scaleY - resolvedMargin.value.top,
  };
}

let observer: ResizeObserver | null = null;
onMounted(() => {
  const target = readElement();
  if (width !== undefined || target === null || typeof ResizeObserver !== "function") return;
  observer = new ResizeObserver((entries) => {
    const entry = entries.at(-1);
    const next = entry?.contentRect.width ?? 0;
    if (next > 0) measuredWidth.value = Math.round(next);
  });
  observer.observe(target);
});
onScopeDispose(() => {
  observer?.disconnect();
  observer = null;
});

chartContext.provide({
  active: computed(() => active.value),
  announce,
  height: resolvedHeight,
  id: baseId,
  innerHeight,
  innerWidth,
  isHidden: (name) => hiddenSet.value.has(name),
  margin: resolvedMargin,
  plotPoint,
  registerSeries: (name, kind) => {
    registered.set(name, kind);
    return () => {
      if (registered.get(name) === kind) registered.delete(name);
    };
  },
  series,
  setActive,
  toggleSeries,
  width: resolvedWidth,
} satisfies ChartContextValue);

type ChartRootSetupExpose = Omit<
  ChartRootExpose,
  "active" | "element" | "height" | "innerHeight" | "innerWidth" | "margin" | "width"
> & {
  readonly active: typeof active;
  readonly element: typeof element;
  readonly height: ComputedRef<number>;
  readonly innerHeight: ComputedRef<number>;
  readonly innerWidth: ComputedRef<number>;
  readonly margin: ComputedRef<ChartMargin>;
  readonly width: ComputedRef<number>;
};

const exposed = {
  active,
  element,
  height: resolvedHeight,
  innerHeight,
  innerWidth,
  margin: resolvedMargin,
  setActive: (point: ChartActivePoint | null) => setActive(point, "programmatic"),
  toggleSeries,
  width: resolvedWidth,
} satisfies ChartRootSetupExpose;

defineExpose(exposed);
</script>

<template>
  <figure
    :id="baseId"
    ref="element"
    data-vize-ui="chart"
    part="root"
    :data-active-series="active?.series"
    :data-width="resolvedWidth"
  >
    <svg
      ref="svg"
      role="group"
      aria-roledescription="chart"
      :aria-labelledby="titleId"
      :aria-describedby="description === undefined ? undefined : descriptionId"
      :width="resolvedWidth"
      :height="resolvedHeight"
      :viewBox="`0 0 ${resolvedWidth} ${resolvedHeight}`"
      data-vize-ui="chart-svg"
      part="svg"
    >
      <title :id="titleId">{{ title }}</title>
      <desc v-if="description !== undefined" :id="descriptionId">{{ description }}</desc>
      <g :transform="plotTransform" data-vize-ui="chart-plot" part="plot">
        <slot v-bind="slotState" />
      </g>
    </svg>
    <slot name="overlay" v-bind="slotState" />
    <slot name="after" v-bind="slotState" />
    <div
      aria-live="polite"
      aria-atomic="true"
      :style="liveRegionStyle"
      data-vize-ui="chart-announcer"
    >
      {{ announcement }}
    </div>
  </figure>
</template>

<style scoped>
/* Headless by design. Only sizing attributes and the hidden live region are inline. */
</style>
