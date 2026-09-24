<script setup lang="ts">
import { computed, shallowRef, useTemplateRef } from "vue";

import {
  getColorChannelRange,
  getColorChannelValue,
  resolveColorChannelSpace,
  setColorChannelValue,
  snapColorChannelValue,
} from "./color-picker-color.ts";
import type { ColorChannel, ColorSpace, ColorValue } from "./color-picker-color.ts";
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
import type { ColorPickerAreaExpose, ColorPickerAreaSlotState } from "./color-picker-types.ts";

const {
  xChannel = "saturation",
  yChannel = "brightness",
  space = "hsb",
  ariaLabel = undefined,
  ariaLabelledby = undefined,
  ariaDescribedby = undefined,
} = defineProps<{
  /**
   * Channel mapped to the horizontal axis (inline start = minimum).
   *
   * @default "saturation"
   */
  readonly xChannel?: ColorChannel;

  /**
   * Channel mapped to the vertical axis (bottom = minimum).
   *
   * @default "brightness"
   */
  readonly yChannel?: ColorChannel;

  /**
   * Color space used to resolve `hue`, `saturation`, and `alpha` channels.
   *
   * @default "hsb"
   */
  readonly space?: ColorSpace;

  /**
   * Accessible name of the 2D slider thumb. Defaults to `"<X> and <Y>"`.
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
  /** Thumb contents. Receives both channel values and thumb position. */
  default(props: ColorPickerAreaSlotState): unknown;
}>();

const context = colorPickerContext.use();
const element = useTemplateRef<HTMLDivElement>("element");
const thumb = useTemplateRef<HTMLDivElement>("thumb");
const activePointer = shallowRef<number | null>(null);
const xSpace = computed(() => resolveColorChannelSpace(xChannel, space));
const ySpace = computed(() => resolveColorChannelSpace(yChannel, space));
const xRange = computed(() => getColorChannelRange(xChannel));
const yRange = computed(() => getColorChannelRange(yChannel));
const xMin = computed<number>(() => xRange.value.min);
const xMax = computed<number>(() => xRange.value.max);
const xValue = computed(() =>
  snapColorChannelValue(
    xChannel,
    getColorChannelValue(context.color.value, xChannel, xSpace.value),
  ),
);
const yValue = computed(() =>
  snapColorChannelValue(
    yChannel,
    getColorChannelValue(context.color.value, yChannel, ySpace.value),
  ),
);
const xPercent = computed(() => valueToPercent(xRange.value, xValue.value));
const yPercent = computed(() => 100 - valueToPercent(yRange.value, yValue.value));
const dragging = computed(() => activePointer.value !== null);

const label = computed(() =>
  ariaLabelledby === undefined ? (ariaLabel ?? context.areaLabel(xChannel, yChannel)) : ariaLabel,
);
const valueText = computed(
  () =>
    `${context.channelValueText(xChannel, xValue.value)}, ${context.channelValueText(yChannel, yValue.value)}`,
);
const areaStyle = computed(() => {
  const style: Record<string, string> = {
    "--vize-ui-color-picker-thumb-x": `${xPercent.value}%`,
    "--vize-ui-color-picker-thumb-y": `${yPercent.value}%`,
    "--vize-ui-color-picker-area-hue": `hsl(${Math.round(context.color.value.hue)} 100% 50%)`,
  };
  if (xChannel === "saturation" && yChannel === "brightness" && xSpace.value === "hsb") {
    style["--vize-ui-color-picker-area-background"] =
      `linear-gradient(to top, #000, transparent), linear-gradient(${
        context.dir.value === "rtl" ? "to left" : "to right"
      }, #fff, hsl(${Math.round(context.color.value.hue)} 100% 50%))`;
  }
  return style;
});
const slotState = computed<ColorPickerAreaSlotState>(() => ({
  color: context.color.value,
  dragging: dragging.value,
  state: context.state.value,
  xChannel,
  xPercent: xPercent.value,
  xValue: xValue.value,
  yChannel,
  yPercent: yPercent.value,
  yValue: yValue.value,
}));

function withChannels(x: number, y: number): ColorValue {
  const withX = setColorChannelValue(
    context.color.value,
    xChannel,
    snapColorChannelValue(xChannel, x),
    xSpace.value,
  );
  return setColorChannelValue(withX, yChannel, snapColorChannelValue(yChannel, y), ySpace.value);
}

function updateFromPointer(event: PointerEvent): void {
  if (element.value === null) return;
  const fraction = pointerFraction(
    readElementRect(element.value),
    event.clientX,
    event.clientY,
    context.dir.value,
  );
  context.setColor(
    withChannels(
      fractionToValue(xRange.value, fraction.x),
      fractionToValue(yRange.value, fraction.y),
    ),
    "area",
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
  if (activePointer.value !== event.pointerId) return;
  updateFromPointer(event);
}

function endDrag(event: PointerEvent, commit: boolean): void {
  if (activePointer.value !== event.pointerId) return;
  activePointer.value = null;
  if (element.value !== null) releasePointer(element.value, event.pointerId);
  if (commit) context.commit("area", event);
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
    dimensions: 2,
    dir: context.dir.value,
    xRange: xRange.value,
    yRange: yRange.value,
  });
  if (intent === null) return;
  event.preventDefault();
  if (!context.editable.value) return;
  const range = intent.axis === "x" ? xRange.value : yRange.value;
  const current = intent.axis === "x" ? xValue.value : yValue.value;
  const target =
    intent.kind === "edge"
      ? range[intent.edge]
      : Math.min(range.max, Math.max(range.min, current + intent.amount));
  const next =
    intent.axis === "x" ? withChannels(target, yValue.value) : withChannels(xValue.value, target);
  if (context.setColor(next, "area", event)) context.commit("area", event);
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

const exposed = { element, focus, thumb } satisfies {
  readonly element: typeof element;
  readonly focus: ColorPickerAreaExpose["focus"];
  readonly thumb: typeof thumb;
};

defineExpose(exposed);
</script>

<template>
  <div
    :id="context.getPartId('area')"
    ref="element"
    data-vize-ui="color-picker-area"
    part="area"
    :data-state="context.state.value"
    :data-dragging="dragging ? 'true' : undefined"
    :data-disabled="context.disabled.value ? 'true' : undefined"
    :data-x-channel="xChannel"
    :data-y-channel="yChannel"
    :style="areaStyle"
    @pointerdown="onPointerdown"
    @pointermove="onPointermove"
    @pointerup="onPointerup"
    @pointercancel="onPointercancel"
    @lostpointercapture="onPointerup"
  >
    <div
      :id="context.getPartId('area-thumb')"
      ref="thumb"
      aria-roledescription="2D slider"
      v-bind="thumbProps"
      :aria-label="label"
      :aria-labelledby="ariaLabelledby"
      :aria-describedby="ariaDescribedby"
      :aria-valuemin="xMin"
      :aria-valuemax="xMax"
      :aria-valuenow="xValue"
      :aria-valuetext="valueText"
      :aria-disabled="context.disabled.value ? 'true' : undefined"
      :aria-readonly="context.readOnly.value ? 'true' : undefined"
      data-vize-ui="color-picker-area-thumb"
      part="thumb"
      :data-state="context.state.value"
      :data-dragging="dragging ? 'true' : undefined"
    >
      <slot v-bind="slotState" />
    </div>
  </div>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
