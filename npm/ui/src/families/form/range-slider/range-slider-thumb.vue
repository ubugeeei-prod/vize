<script setup lang="ts">
import { computed, onScopeDispose, useTemplateRef, watch } from "vue";

import { rangeSliderContext } from "./range-slider-context.ts";
import { getRangeSliderThumbBounds, rangeSliderPercent } from "./range-slider-state.ts";
import type {
  RangeSliderThumbProps,
  RangeSliderThumbSlotState,
  RangeSliderThumbStyle,
} from "./range-slider-types.ts";

const { index, ariaLabel = undefined } = defineProps<RangeSliderThumbProps>();

defineSlots<{
  /** Thumb contents, for example a value tooltip, with the thumb state. */
  default?(props: RangeSliderThumbSlotState): unknown;
}>();

const context = rangeSliderContext.use();
const element = useTemplateRef<HTMLSpanElement>("element");
const value = computed(() => context.values.value[index] ?? context.bounds.value.min);
const limits = computed(() =>
  getRangeSliderThumbBounds(context.values.value, index, context.bounds.value),
);
const percent = computed(() => rangeSliderPercent(value.value, context.bounds.value));
const active = computed(() => context.activeThumb.value === index);
const valueText = computed(() => context.getValueText.value?.(value.value, index));
const thumbStyle = computed<RangeSliderThumbStyle>(() => ({
  "--vize-range-slider-thumb-percent": `${percent.value}%`,
}));
const slotState = computed<RangeSliderThumbSlotState>(() => ({
  index,
  value: value.value,
  percent: percent.value,
  min: limits.value.min,
  max: limits.value.max,
  active: active.value,
  disabled: context.disabled.value,
}));

watch(element, (next) => context.registerThumb(index, next), { flush: "sync", immediate: true });
onScopeDispose(() => context.registerThumb(index, null));

function keyTarget(key: string): number | undefined {
  const bounds = context.bounds.value;
  const horizontal = context.orientation.value === "horizontal";
  const forward = horizontal && context.direction.value === "rtl" ? -1 : 1;
  if (key === "ArrowUp") return value.value + bounds.step;
  if (key === "ArrowDown") return value.value - bounds.step;
  if (key === "ArrowRight") return value.value + bounds.step * forward;
  if (key === "ArrowLeft") return value.value - bounds.step * forward;
  if (key === "PageUp") return value.value + bounds.largeStep;
  if (key === "PageDown") return value.value - bounds.largeStep;
  if (key === "Home") return limits.value.min;
  if (key === "End") return limits.value.max;
  return undefined;
}

function onKeydown(event: KeyboardEvent): void {
  if (context.disabled.value || event.altKey || event.metaKey || event.ctrlKey) return;
  const target = keyTarget(event.key);
  if (target === undefined) return;
  event.preventDefault();
  context.setThumb(index, target);
  context.commit(index, "keyboard");
}
</script>

<template>
  <span
    ref="element"
    role="slider"
    tabindex="0"
    :aria-label="ariaLabel"
    :aria-labelledby="ariaLabel === undefined ? context.ariaLabelledby.value : undefined"
    :aria-describedby="context.ariaDescribedby.value"
    :aria-errormessage="
      context.ariaInvalid.value === undefined ? undefined : context.ariaErrormessage.value
    "
    :aria-invalid="context.ariaInvalid.value"
    :aria-valuenow="value"
    :aria-valuemin="limits.min"
    :aria-valuemax="limits.max"
    :aria-valuetext="valueText"
    :aria-orientation="context.orientation.value"
    :aria-disabled="context.disabled.value ? 'true' : undefined"
    part="thumb"
    data-vize-ui="range-slider-thumb"
    :data-index="index"
    :data-active="active ? 'true' : undefined"
    :data-disabled="context.disabled.value ? 'true' : undefined"
    :style="thumbStyle"
    @keydown="onKeydown"
  >
    <slot v-bind="slotState" />
  </span>
</template>

<style scoped>
/* Headless by design. Position with --vize-range-slider-thumb-percent. */
</style>
