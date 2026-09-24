<script setup lang="ts">
import { computed, nextTick, shallowRef, useTemplateRef, watch } from "vue";
import type { ComputedRef } from "vue";

import { useControllableState } from "../../foundations/controllable-state/controllable-state.ts";
import { useDeterministicId } from "../../foundations/id/deterministic-id.ts";
import {
  angleToValue,
  normalizeRotaryBounds,
  normalizeRotarySweep,
  snapRotaryValue,
  valueToAngle,
} from "./knob-geometry.ts";
import { useRotaryDrag } from "./knob-pointer.ts";
import type {
  KnobAriaInvalid,
  KnobChangeSource,
  KnobExpose,
  KnobSlotState,
  KnobState,
  KnobStyle,
} from "./knob-types.ts";

const {
  id = undefined,
  name = undefined,
  modelValue = undefined,
  defaultValue = undefined,
  min = 0,
  max = 100,
  step = 1,
  largeStep = undefined,
  startAngle = -135,
  endAngle = 135,
  allowWheel = false,
  disabled = false,
  readOnly = false,
  getValueText = undefined,
  ariaLabel = undefined,
  ariaLabelledby = undefined,
  ariaDescribedby = undefined,
  ariaErrormessage = undefined,
  ariaInvalid = false,
} = defineProps<{
  /**
   * Element id. `null` and `undefined` select a deterministic fallback.
   *
   * @default undefined
   */
  readonly id?: string | null;

  /**
   * Native form field name; a hidden input submits the value.
   *
   * @default undefined
   */
  readonly name?: string;

  /**
   * Controlled value. `undefined` selects uncontrolled behavior.
   *
   * @default undefined
   */
  readonly modelValue?: number;

  /**
   * Initial uncontrolled value, also restored by form reset.
   *
   * @default min
   */
  readonly defaultValue?: number;

  /**
   * Lower bound.
   *
   * @default 0
   */
  readonly min?: number;

  /**
   * Upper bound (repaired to `min + 1` when not greater than `min`).
   *
   * @default 100
   */
  readonly max?: number;

  /**
   * Positive step.
   *
   * @default 1
   */
  readonly step?: number;

  /**
   * Positive step for Page Up and Page Down.
   *
   * @default step * 10
   */
  readonly largeStep?: number;

  /**
   * Indicator angle of `min`, in degrees clockwise from 12 o'clock.
   *
   * @default -135
   */
  readonly startAngle?: number;

  /**
   * Indicator angle of `max`; the sweep must be positive and at most 360°.
   *
   * @default 135
   */
  readonly endAngle?: number;

  /**
   * Opt in to wheel stepping while focused.
   *
   * @default false
   */
  readonly allowWheel?: boolean;

  /**
   * Disable interaction and form submission.
   *
   * @default false
   */
  readonly disabled?: boolean;

  /**
   * Keep focus and form value while preventing user changes.
   *
   * @default false
   */
  readonly readOnly?: boolean;

  /**
   * Formats `aria-valuetext`, for example `(v) => \`${v} dB\``.
   *
   * @default undefined
   */
  readonly getValueText?: (value: number) => string;

  /**
   * Accessible name when no label or `aria-labelledby` supplies one.
   *
   * @default undefined
   */
  readonly ariaLabel?: string;

  /**
   * Ids that label the knob.
   *
   * @default undefined
   */
  readonly ariaLabelledby?: string;

  /**
   * Ids that describe the knob.
   *
   * @default undefined
   */
  readonly ariaDescribedby?: string;

  /**
   * Id of the validation error message used while invalid.
   *
   * @default undefined
   */
  readonly ariaErrormessage?: string;

  /**
   * Invalid state announced to assistive technology.
   *
   * @default false
   */
  readonly ariaInvalid?: KnobAriaInvalid;
}>();

const emit = defineEmits<{
  /** Fired for every requested value, including drag frames. */
  "update:modelValue": [value: number];

  /** Fired once a change is committed (key press, pointer release, wheel, or API). */
  change: [value: number, source: KnobChangeSource];
}>();

defineSlots<{
  /** Renders the dial face and indicator with the angle and value. */
  default?(props: KnobSlotState): unknown;
}>();

const element = useTemplateRef<HTMLSpanElement>("element");
const controlId = useDeterministicId({ id: () => id, hint: "knob" });
const bounds = computed(() => normalizeRotaryBounds({ min, max, step, largeStep }));
const sweep = computed(() => normalizeRotarySweep({ startAngle, endAngle }));
const state = useControllableState<number>({
  value: () => (modelValue === undefined ? undefined : snapRotaryValue(modelValue, bounds.value)),
  defaultValue: () => snapRotaryValue(defaultValue ?? bounds.value.min, bounds.value),
  onChange: (value) => emit("update:modelValue", value),
});
const value = computed(() => snapRotaryValue(state.value.value, bounds.value));
const angle = computed(() => valueToAngle(value.value, bounds.value, sweep.value));
const percent = computed(
  () => ((value.value - bounds.value.min) / (bounds.value.max - bounds.value.min)) * 100,
);
const dragging = shallowRef(false);
const committed = shallowRef(value.value);
const ariaInvalidValue = computed(() => {
  if (ariaInvalid === false) return undefined;
  return ariaInvalid === true ? "true" : ariaInvalid;
});
const dataState = computed<KnobState>(() => {
  if (disabled) return "disabled";
  if (readOnly) return "readonly";
  return dragging.value ? "dragging" : "idle";
});
const rootStyle = computed<KnobStyle>(() => ({
  "--vize-knob-angle": `${angle.value}deg`,
  "--vize-knob-percent": `${percent.value}%`,
}));
const editable = () => !disabled && !readOnly;

function request(next: number): boolean {
  return state.set(snapRotaryValue(next, bounds.value));
}

function commit(source: KnobChangeSource): void {
  if (committed.value === value.value) return;
  committed.value = value.value;
  emit("change", value.value, source);
}

function keyTarget(key: string): number | undefined {
  const { min: low, max: high, step: size, largeStep: large } = bounds.value;
  if (key === "ArrowUp" || key === "ArrowRight") return value.value + size;
  if (key === "ArrowDown" || key === "ArrowLeft") return value.value - size;
  if (key === "PageUp") return value.value + large;
  if (key === "PageDown") return value.value - large;
  if (key === "Home") return low;
  if (key === "End") return high;
  return undefined;
}

function onKeydown(event: KeyboardEvent): void {
  if (event.altKey || event.ctrlKey || event.metaKey) return;
  const target = keyTarget(event.key);
  if (target === undefined) return;
  event.preventDefault();
  if (!editable()) return;
  request(target);
  commit("keyboard");
}

const drag = useRotaryDrag({
  enabled: editable,
  onAngle: (pointer) => {
    dragging.value = true;
    request(angleToValue(pointer, bounds.value, sweep.value));
  },
  onEnd: () => {
    dragging.value = false;
    commit("pointer");
  },
});

function onWheel(event: WheelEvent): void {
  if (
    !editable() ||
    event.deltaY === 0 ||
    element.value?.ownerDocument.activeElement !== element.value
  )
    return;
  event.preventDefault();
  request(value.value + (event.deltaY < 0 ? bounds.value.step : -bounds.value.step));
  commit("wheel");
}

// Non-passive wheel listener, attached only when opted in.
watch(
  [element, () => allowWheel],
  ([target, enabled], _previous, onCleanup) => {
    if (target === null || !enabled) return;
    target.addEventListener("wheel", onWheel, { passive: false });
    onCleanup(() => target.removeEventListener("wheel", onWheel));
  },
  { flush: "post", immediate: true },
);

watch(
  element,
  (target, _previous, onCleanup) => {
    const owner = target?.closest("form");
    if (owner === null || owner === undefined) return;
    const onReset = () => {
      if (!state.controlled.value) state.reset();
      void nextTick(() => {
        committed.value = value.value;
      });
    };
    owner.addEventListener("reset", onReset);
    onCleanup(() => owner.removeEventListener("reset", onReset));
  },
  { flush: "post", immediate: true },
);

const slotState = computed<KnobSlotState>(() => ({
  value: value.value,
  angle: angle.value,
  percent: percent.value,
  min: bounds.value.min,
  max: bounds.value.max,
  dragging: dragging.value,
  disabled,
  state: dataState.value,
}));

type KnobSetupExpose = Omit<KnobExpose, keyof KnobSlotState | "element"> & {
  readonly [Key in keyof KnobSlotState]: ComputedRef<KnobSlotState[Key]>;
} & { readonly element: typeof element };

function field<Key extends keyof KnobSlotState>(key: Key): ComputedRef<KnobSlotState[Key]> {
  return computed(() => slotState.value[key]);
}

const exposed = {
  value: field("value"),
  angle: field("angle"),
  percent: field("percent"),
  min: field("min"),
  max: field("max"),
  dragging: field("dragging"),
  disabled: field("disabled"),
  state: field("state"),
  element,
  focus: (options?: FocusOptions) => element.value?.focus(options),
  setValue: (next: number) => {
    const changed = request(next);
    commit("api");
    return changed;
  },
  reset: () => {
    const changed = state.reset();
    committed.value = value.value;
    return changed;
  },
} satisfies KnobSetupExpose;

defineExpose(exposed);
</script>

<template>
  <span
    :id="controlId"
    ref="element"
    role="slider"
    tabindex="0"
    :aria-valuenow="value"
    :aria-valuemin="bounds.min"
    :aria-valuemax="bounds.max"
    :aria-valuetext="getValueText?.(value)"
    :aria-label="ariaLabel"
    :aria-labelledby="ariaLabelledby"
    :aria-describedby="ariaDescribedby"
    :aria-errormessage="ariaInvalidValue === undefined ? undefined : ariaErrormessage"
    :aria-invalid="ariaInvalidValue"
    :aria-disabled="disabled ? 'true' : undefined"
    :aria-readonly="readOnly ? 'true' : undefined"
    part="root"
    data-vize-ui="knob"
    :data-state="dataState"
    :style="rootStyle"
    @keydown="onKeydown"
    @pointerdown="drag.onPointerdown"
  >
    <input v-if="name !== undefined" type="hidden" :name :value="String(value)" :disabled />
    <slot v-bind="slotState" />
  </span>
</template>

<style scoped>
/* Headless by design. Rotate an indicator with --vize-knob-angle. */
</style>
