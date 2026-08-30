<script setup lang="ts">
import { computed, nextTick, useTemplateRef, watch, watchEffect } from "vue";
import type { ComputedRef } from "vue";

import { useControllableState } from "../../../controllable-state.ts";
import { useDeterministicId } from "../../foundations/id/deterministic-id.ts";
import {
  SLIDER_DEFAULT_MAX,
  SLIDER_DEFAULT_MIN,
  SLIDER_DEFAULT_STEP,
  getSliderState,
} from "./slider-state.ts";
import type {
  SliderEmits,
  SliderExpose,
  SliderProps,
  SliderSlotState,
  SliderStyle,
} from "./slider-types.ts";

const {
  id = undefined,
  name = undefined,
  modelValue = undefined,
  defaultValue = undefined,
  min = SLIDER_DEFAULT_MIN,
  max = SLIDER_DEFAULT_MAX,
  step = SLIDER_DEFAULT_STEP,
  disabled = false,
  readOnly = false,
  required = false,
  orientation = "horizontal",
  dir = "ltr",
  ariaLabel = undefined,
  ariaLabelledby = undefined,
  ariaDescribedby = undefined,
  ariaErrormessage = undefined,
  ariaValueText = undefined,
  ariaInvalid = false,
} = defineProps<SliderProps>();

const emit = defineEmits<SliderEmits>();

defineSlots<{
  /** Renders optional marks or output with the normalized Slider state. */
  default(props: SliderSlotState): unknown;
}>();

const root = useTemplateRef<HTMLSpanElement>("root");
const element = useTemplateRef<HTMLInputElement>("element");
const controlId = useDeterministicId({ id: () => id, hint: "slider" });
const ariaInvalidValue = computed(() => {
  if (ariaInvalid === false) return undefined;
  return ariaInvalid === true ? "true" : ariaInvalid;
});
const state = useControllableState({
  value: () => modelValue,
  defaultValue: () => defaultValue ?? min,
  onChange: (value) => emit("update:modelValue", value),
});
const rawValue = state.value;
const slider = computed(() =>
  getSliderState({
    value: rawValue.value,
    min,
    max,
    step,
    disabled,
    readOnly,
    required,
    invalid: ariaInvalidValue.value !== undefined,
    orientation,
    direction: dir,
  }),
);
const currentValue = computed(() => slider.value.value);
const currentMin = computed(() => slider.value.min);
const currentMax = computed(() => slider.value.max);
const currentStep = computed(() => slider.value.step);
const percent = computed(() => slider.value.percent);
const orientationState = computed(() => slider.value.orientation);
const directionState = computed(() => slider.value.direction);
const disabledState = computed(() => slider.value.disabled);
const readOnlyState = computed(() => slider.value.readOnly);
const requiredState = computed(() => slider.value.required);
const invalidState = computed(() => slider.value.invalid);
const dataState = computed(() => slider.value.state);
const sliderStyle = computed<SliderStyle>(() => ({
  "--vize-slider-value": String(currentValue.value),
  "--vize-slider-min": String(currentMin.value),
  "--vize-slider-max": String(currentMax.value),
  "--vize-slider-step": String(currentStep.value),
  "--vize-slider-percent": `${percent.value}%`,
}));
const slotState = computed<SliderSlotState>(() => ({
  value: currentValue.value,
  min: currentMin.value,
  max: currentMax.value,
  step: currentStep.value,
  percent: percent.value,
  orientation: orientationState.value,
  direction: directionState.value,
  disabled: disabledState.value,
  readOnly: readOnlyState.value,
  required: requiredState.value,
  invalid: invalidState.value,
  state: dataState.value,
}));
const intrinsicProps = computed(() => ({ style: sliderStyle.value }));
const readonlyKeys = new Set([
  "ArrowDown",
  "ArrowLeft",
  "ArrowRight",
  "ArrowUp",
  "End",
  "Home",
  "PageDown",
  "PageUp",
]);

function normalizedValue(value: number): number {
  return getSliderState({ value, min, max, step }).value;
}

function syncNativeValue(): void {
  if (element.value === null) return;
  const next = String(currentValue.value);
  if (element.value.value !== next) element.value.value = next;
}

watchEffect(syncNativeValue);

watch(
  element,
  (input, _previous, onCleanup) => {
    const form = input?.form;
    if (form === undefined || form === null) return;
    const onReset = () => {
      if (!state.controlled.value) state.reset();
      void nextTick(syncNativeValue);
    };
    form.addEventListener("reset", onReset);
    onCleanup(() => form.removeEventListener("reset", onReset));
  },
  { flush: "post", immediate: true },
);

function readSliderValue(event: Event): number | undefined {
  if (!(event.currentTarget instanceof HTMLInputElement)) return undefined;
  return normalizedValue(Number(event.currentTarget.value));
}

function onInput(event: Event): void {
  const next = readSliderValue(event);
  if (next === undefined) return;
  if (readOnlyState.value) {
    event.preventDefault();
    void nextTick(syncNativeValue);
    return;
  }
  state.set(next);
  emit("input", next, event);
  void nextTick(syncNativeValue);
}

function onChange(event: Event): void {
  const next = readSliderValue(event);
  if (next === undefined) return;
  if (readOnlyState.value) {
    event.preventDefault();
    void nextTick(syncNativeValue);
    return;
  }
  state.set(next);
  emit("change", next, event);
  void nextTick(syncNativeValue);
}

function onKeydown(event: KeyboardEvent): void {
  if (!readOnlyState.value || !readonlyKeys.has(event.key)) return;
  event.preventDefault();
  void nextTick(syncNativeValue);
}

function onPointerdown(event: PointerEvent): void {
  if (!readOnlyState.value) return;
  event.preventDefault();
  void nextTick(syncNativeValue);
}

function onClick(event: MouseEvent): void {
  if (!readOnlyState.value) return;
  event.preventDefault();
  void nextTick(syncNativeValue);
}

/** Move focus to the native range input. */
function focus(options?: FocusOptions): void {
  element.value?.focus(options);
}

function setValue(next: number): boolean {
  const changed = state.set(normalizedValue(next));
  void nextTick(syncNativeValue);
  return changed;
}

function stepDelta(steps: number | undefined): number {
  const count = typeof steps === "number" && Number.isFinite(steps) ? steps : 1;
  const stepSize = currentStep.value === "any" ? SLIDER_DEFAULT_STEP : currentStep.value;
  return stepSize * count;
}

function stepUp(steps?: number): boolean {
  return setValue(currentValue.value + stepDelta(steps));
}

function stepDown(steps?: number): boolean {
  return setValue(currentValue.value - stepDelta(steps));
}

type SliderSetupExpose = Omit<
  SliderExpose,
  | "direction"
  | "disabled"
  | "element"
  | "invalid"
  | "max"
  | "min"
  | "orientation"
  | "percent"
  | "readOnly"
  | "required"
  | "root"
  | "state"
  | "step"
  | "value"
> & {
  readonly direction: ComputedRef<SliderExpose["direction"]>;
  readonly disabled: ComputedRef<SliderExpose["disabled"]>;
  readonly element: typeof element;
  readonly invalid: ComputedRef<SliderExpose["invalid"]>;
  readonly max: ComputedRef<SliderExpose["max"]>;
  readonly min: ComputedRef<SliderExpose["min"]>;
  readonly orientation: ComputedRef<SliderExpose["orientation"]>;
  readonly percent: ComputedRef<SliderExpose["percent"]>;
  readonly readOnly: ComputedRef<SliderExpose["readOnly"]>;
  readonly required: ComputedRef<SliderExpose["required"]>;
  readonly root: typeof root;
  readonly state: ComputedRef<SliderExpose["state"]>;
  readonly step: ComputedRef<SliderExpose["step"]>;
  readonly value: ComputedRef<SliderExpose["value"]>;
};

const exposed = {
  direction: directionState,
  disabled: disabledState,
  element,
  focus,
  invalid: invalidState,
  max: currentMax,
  min: currentMin,
  orientation: orientationState,
  percent,
  readOnly: readOnlyState,
  required: requiredState,
  reset: state.reset,
  root,
  setValue,
  state: dataState,
  step: currentStep,
  stepDown,
  stepUp,
  value: currentValue,
} satisfies SliderSetupExpose;

defineExpose(exposed);
</script>

<template>
  <span
    ref="root"
    data-vize-ui="slider"
    part="root"
    :data-state="dataState"
    :data-orientation="orientationState"
    :data-dir="directionState"
    :data-value="currentValue"
    :data-min="currentMin"
    :data-max="currentMax"
    :data-step="currentStep"
    :data-percent="percent"
    :data-disabled="disabledState ? 'true' : undefined"
    :data-readonly="readOnlyState ? 'true' : undefined"
    :data-required="requiredState ? 'true' : undefined"
    :data-invalid="invalidState ? 'true' : undefined"
    v-bind="intrinsicProps"
  >
    <input
      :id="controlId"
      ref="element"
      type="range"
      :name
      :value="currentValue"
      :min="currentMin"
      :max="currentMax"
      :step="currentStep"
      :disabled="disabledState"
      :required="requiredState"
      :dir="directionState"
      :aria-label="ariaLabel"
      :aria-labelledby="ariaLabelledby"
      :aria-describedby="ariaDescribedby"
      :aria-errormessage="ariaInvalidValue === undefined ? undefined : ariaErrormessage"
      :aria-invalid="ariaInvalidValue"
      :aria-orientation="orientationState"
      :aria-readonly="readOnlyState ? 'true' : undefined"
      :aria-valuetext="ariaValueText"
      :orient="orientationState === 'vertical' ? 'vertical' : undefined"
      data-vize-ui="slider-input"
      part="control"
      :data-state="dataState"
      :data-orientation="orientationState"
      :data-dir="directionState"
      :data-value="currentValue"
      :data-min="currentMin"
      :data-max="currentMax"
      :data-step="currentStep"
      :data-percent="percent"
      @input="onInput"
      @change="onChange"
      @keydown="onKeydown"
      @pointerdown="onPointerdown"
      @click="onClick"
    />
    <slot v-bind="slotState" />
  </span>
</template>

<style scoped>
/* Headless by design. Native range styling remains entirely consumer-owned. */
</style>
