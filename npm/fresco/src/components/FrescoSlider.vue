<script setup lang="ts">
import { computed, useTemplateRef } from "vue";

import { useBoxMetrics, type MetricTarget } from "../composables/useBoxMetrics.js";
import { useInput } from "../composables/useInput.js";
import { useMouse } from "../composables/useMouse.js";
import type { MouseEvent } from "../protocol.js";
import { Box } from "./Box.js";
import {
  normalizeSliderRange,
  normalizeSliderValue,
  sliderValueAtRatio,
  type SliderProps,
} from "./slider-types.js";
import { Text } from "./Text.js";

const {
  modelValue = 0,
  min = 0,
  max = 100,
  step = 1,
  pageStep,
  length = 20,
  orientation = "horizontal",
  focused = false,
  disabled = false,
  filled = "━",
  empty = "─",
  thumb = "●",
  fg = "gray",
  filledFg = "green",
  thumbFg = "cyan",
  label = "Slider",
  formatValue = (value: number) => String(value),
} = defineProps<SliderProps>();

const emit = defineEmits<{
  "update:modelValue": [value: number];
  "change": [value: number];
}>();

defineOptions({ name: "FrescoSlider" });

const trackRef = useTemplateRef<MetricTarget>("trackRef");
const metrics = useBoxMetrics(trackRef);
const range = computed(() => normalizeSliderRange(min, max, step));
const value = computed(() => normalizeSliderValue(modelValue, range.value));
const cellCount = computed(() => {
  const cells = Math.floor(length);
  return Number.isSafeInteger(cells) && cells > 0 ? cells : 1;
});
const thumbIndex = computed(() => {
  const span = range.value.max - range.value.min;
  const ratio = span === 0 ? 0 : (value.value - range.value.min) / span;
  return Math.round(ratio * (cellCount.value - 1));
});
const cells = computed(() =>
  Array.from({ length: cellCount.value }, (_, index) => ({
    index,
    content: index === thumbIndex.value ? thumb : index < thumbIndex.value ? filled : empty,
    fg: index === thumbIndex.value ? thumbFg : index < thumbIndex.value ? filledFg : fg,
  })),
);
const isActive = computed(() => focused && !disabled);
const effectivePageStep = computed(() => {
  const requested = pageStep ?? range.value.step * 10;
  return Number.isFinite(requested) && requested > 0 ? requested : range.value.step * 10;
});
const ariaState = computed(() => ({
  orientation,
  valueMin: range.value.min,
  valueMax: range.value.max,
  valueNow: value.value,
}));
const accessibleLabel = computed(() => `${label}: ${formatValue(value.value)}`);
const accessibilityProps = computed(() => ({
  ariaRole: "slider" as const,
  ariaLabel: accessibleLabel.value,
  ariaState: ariaState.value,
}));

function commit(candidate: number): void {
  if (disabled) return;
  const next = normalizeSliderValue(candidate, range.value);
  if (Object.is(next, value.value)) return;
  emit("update:modelValue", next);
  emit("change", next);
}

function adjust(delta: number): void {
  commit(value.value + delta);
}

useInput({
  isActive,
  onArrow: (direction) => {
    if (direction === "left" || direction === "down") adjust(-range.value.step);
    if (direction === "right" || direction === "up") adjust(range.value.step);
  },
  onKey: (key) => {
    if (key === "home") commit(range.value.min);
    if (key === "end") commit(range.value.max);
    if (key === "pageup") adjust(effectivePageStep.value);
    if (key === "pagedown") adjust(-effectivePageStep.value);
  },
});

function containsPoint(event: MouseEvent): boolean {
  return (
    metrics.hasMeasured &&
    event.x >= metrics.left &&
    event.x < metrics.left + metrics.width &&
    event.y >= metrics.top &&
    event.y < metrics.top + metrics.height
  );
}

function handleMouse(event: MouseEvent): void {
  metrics.refresh();
  if (!containsPoint(event)) return;
  if (event.kind === "scroll-up" || event.kind === "scroll-right") {
    adjust(range.value.step);
    return;
  }
  if (event.kind === "scroll-down" || event.kind === "scroll-left") {
    adjust(-range.value.step);
    return;
  }
  if (event.kind !== "down" && event.kind !== "drag") return;

  const horizontal = orientation === "horizontal";
  const extent = Math.max(1, (horizontal ? metrics.width : metrics.height) - 1);
  const position = horizontal ? event.x - metrics.left : event.y - metrics.top;
  const ratio = horizontal ? position / extent : 1 - position / extent;
  commit(sliderValueAtRatio(ratio, range.value));
}

useMouse(handleMouse, { isActive });
</script>

<template>
  <Box
    ref="trackRef"
    :flex-direction="orientation === 'horizontal' ? 'row' : 'column'"
    :width="orientation === 'horizontal' ? cellCount : 1"
    :height="orientation === 'vertical' ? cellCount : 1"
    v-bind="accessibilityProps"
  >
    <Text
      v-for="cell in cells"
      :key="cell.index"
      :content="cell.content"
      :fg="cell.fg"
      :bold="focused && cell.index === thumbIndex"
      :dim="disabled"
      aria-hidden
    />
  </Box>
</template>
