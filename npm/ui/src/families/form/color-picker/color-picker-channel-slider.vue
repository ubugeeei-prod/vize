<script setup lang="ts">
import { computed, shallowRef, useTemplateRef } from "vue";

import {
  formatColorChannelValue,
  getColorChannelGradient,
  getColorChannelLabel,
  getColorChannelRange,
  getColorChannelValue,
  resolveColorChannelSpace,
  setColorChannelValue,
  snapColorChannelValue,
} from "./color-picker-color.ts";
import type { ColorChannel, ColorSpace } from "./color-picker-color.ts";
import { colorPickerContext } from "./color-picker-context.ts";
import {
  capturePointer,
  fractionToValue,
  pointerFraction,
  readElementRect,
  releasePointer,
  resolveKeyIntent,
  valueToPercent,
} from "./color-picker-interaction.ts";
import type {
  ColorPickerChannelSliderExpose,
  ColorPickerChannelSliderSlotState,
  ColorPickerOrientation,
} from "./color-picker-types.ts";

const {
  channel,
  space = "hsb",
  orientation = "horizontal",
  step = undefined,
  ariaLabel = undefined,
  ariaLabelledby = undefined,
  ariaDescribedby = undefined,
} = defineProps<{
  /** Channel edited by this slider, e.g. `"hue"` or `"alpha"`. @default required */
  readonly channel: ColorChannel;

  /**
   * Color space used to resolve `hue`, `saturation`, and `alpha`.
   *
   * @default "hsb"
   */
  readonly space?: ColorSpace;

  /**
   * Layout axis. Vertical sliders place the minimum at the bottom.
   *
   * @default "horizontal"
   */
  readonly orientation?: ColorPickerOrientation;

  /**
   * Arrow-key and snapping increment. Defaults to the channel's natural step.
   *
   * @default undefined
   */
  readonly step?: number;

  /**
   * Accessible name. Defaults to the English channel label, e.g. `"Hue"`.
   *
   * @default undefined
   */
  readonly ariaLabel?: string;

  /**
   * Space-separated ids that label the thumb.
   *
   * @default undefined
   */
  readonly ariaLabelledby?: string;

  /**
   * Space-separated ids that describe the thumb.
   *
   * @default undefined
   */
  readonly ariaDescribedby?: string;
}>();

defineSlots<{
  /** Thumb contents. Receives the channel value and thumb position. */
  default(props: ColorPickerChannelSliderSlotState): unknown;
}>();

const context = colorPickerContext.use();
const element = useTemplateRef<HTMLDivElement>("element");
const thumb = useTemplateRef<HTMLDivElement>("thumb");
const activePointer = shallowRef<number | null>(null);
const resolvedSpace = computed(() => resolveColorChannelSpace(channel, space));
const range = computed(() => {
  const base = getColorChannelRange(channel);
  if (step === undefined || !Number.isFinite(step) || step <= 0) return base;
  return { ...base, step, pageStep: Math.max(step, base.pageStep) };
});
const min = computed<number>(() => range.value.min);
const max = computed<number>(() => range.value.max);
const value = computed(() =>
  snapColorChannelValue(
    channel,
    getColorChannelValue(context.color.value, channel, resolvedSpace.value),
    range.value.step,
  ),
);
const percent = computed(() => valueToPercent(range.value, value.value));
const dragging = computed(() => activePointer.value !== null);

const label = computed(() =>
  ariaLabelledby === undefined ? (ariaLabel ?? getColorChannelLabel(channel)) : ariaLabel,
);
const gradientDirection = computed(() => {
  if (orientation === "vertical") return "to top";
  return context.dir.value === "rtl" ? "to left" : "to right";
});
const sliderStyle = computed(() => ({
  "--vize-ui-color-picker-thumb-percent": `${percent.value}%`,
  "--vize-ui-color-picker-track-background": getColorChannelGradient(
    context.color.value,
    channel,
    resolvedSpace.value,
    gradientDirection.value,
  ),
}));
const slotState = computed<ColorPickerChannelSliderSlotState>(() => ({
  channel,
  color: context.color.value,
  dragging: dragging.value,
  max: range.value.max,
  min: range.value.min,
  orientation,
  percent: percent.value,
  space: resolvedSpace.value,
  state: context.state.value,
  value: value.value,
}));

function setValue(next: number, event: Event): boolean {
  const snapped = snapColorChannelValue(channel, next, range.value.step);
  return context.setColor(
    setColorChannelValue(context.color.value, channel, snapped, resolvedSpace.value),
    "channel",
    event,
  );
}

function updateFromPointer(event: PointerEvent): void {
  if (element.value === null) return;
  const fraction = pointerFraction(
    readElementRect(element.value),
    event.clientX,
    event.clientY,
    context.dir.value,
  );
  setValue(
    fractionToValue(range.value, orientation === "vertical" ? fraction.y : fraction.x),
    event,
  );
}

function onPointerdown(event: PointerEvent): void {
  if (!context.editable.value || event.button !== 0 || element.value === null) return;
  event.preventDefault();
  activePointer.value = event.pointerId;
  capturePointer(element.value, event.pointerId);
  thumb.value?.focus({ preventScroll: true });
  updateFromPointer(event);
}

function onPointermove(event: PointerEvent): void {
  if (activePointer.value === event.pointerId) updateFromPointer(event);
}

function endDrag(event: PointerEvent, commit: boolean): void {
  if (activePointer.value !== event.pointerId) return;
  activePointer.value = null;
  if (element.value !== null) releasePointer(element.value, event.pointerId);
  if (commit) context.commit("channel", event);
}

function onPointerup(event: PointerEvent): void {
  endDrag(event, true);
}

function onPointercancel(event: PointerEvent): void {
  endDrag(event, false);
}

function onKeydown(event: KeyboardEvent): void {
  if (context.disabled.value) return;
  const intent = resolveKeyIntent(event, {
    dimensions: 1,
    dir: context.dir.value,
    orientation,
    xRange: range.value,
  });
  if (intent === null) return;
  event.preventDefault();
  if (!context.editable.value) return;
  const target =
    intent.kind === "edge"
      ? range.value[intent.edge]
      : Math.min(range.value.max, Math.max(range.value.min, value.value + intent.amount));
  if (setValue(target, event)) context.commit("channel", event);
}

// Role, tabindex, and keyboard handling are bound together; disabled thumbs leave the tab order.
const thumbProps = computed<{
  readonly role: "slider";
  readonly tabindex: -1 | 0;
  readonly onKeydown: (event: KeyboardEvent) => void;
}>(() => ({
  role: "slider",
  tabindex: context.disabled.value ? -1 : 0,
  onKeydown,
}));

function focus(options?: FocusOptions): void {
  thumb.value?.focus(options);
}

const exposed = { element, focus, thumb, value } satisfies {
  readonly element: typeof element;
  readonly focus: ColorPickerChannelSliderExpose["focus"];
  readonly thumb: typeof thumb;
  readonly value: typeof value;
};

defineExpose(exposed);
</script>

<template>
  <div
    :id="context.getPartId(`channel-${channel}`)"
    ref="element"
    data-vize-ui="color-picker-channel-slider"
    part="track"
    :data-state="context.state.value"
    :data-channel="channel"
    :data-orientation="orientation"
    :data-dragging="dragging ? 'true' : undefined"
    :data-disabled="context.disabled.value ? 'true' : undefined"
    :style="sliderStyle"
    @pointerdown="onPointerdown"
    @pointermove="onPointermove"
    @pointerup="onPointerup"
    @pointercancel="onPointercancel"
    @lostpointercapture="onPointerup"
  >
    <div
      :id="context.getPartId(`channel-${channel}-thumb`)"
      ref="thumb"
      v-bind="thumbProps"
      :aria-label="label"
      :aria-labelledby="ariaLabelledby"
      :aria-describedby="ariaDescribedby"
      :aria-orientation="orientation"
      :aria-valuemin="min"
      :aria-valuemax="max"
      :aria-valuenow="value"
      :aria-valuetext="formatColorChannelValue(channel, value)"
      :aria-disabled="context.disabled.value ? 'true' : undefined"
      :aria-readonly="context.readOnly.value ? 'true' : undefined"
      data-vize-ui="color-picker-channel-thumb"
      part="thumb"
      :data-state="context.state.value"
      :data-channel="channel"
    >
      <slot v-bind="slotState" />
    </div>
  </div>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
