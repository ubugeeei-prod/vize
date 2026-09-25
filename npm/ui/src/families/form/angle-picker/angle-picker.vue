<script setup lang="ts">
import { computed, shallowRef, useTemplateRef, watch } from "vue";
import type { ComputedRef } from "vue";

import { useControllableState } from "../../foundations/controllable-state/controllable-state.ts";
import { useDeterministicId } from "../../foundations/id/deterministic-id.ts";
import { snapAngle, wrapAngle } from "../knob/knob-geometry.ts";
import { useRotaryDrag } from "../knob/knob-pointer.ts";
import type {
  AnglePickerAriaInvalid,
  AnglePickerChangeSource,
  AnglePickerExpose,
  AnglePickerSlotState,
  AnglePickerState,
  AnglePickerStyle,
} from "./angle-picker-types.ts";

const {
  id = undefined,
  name = undefined,
  modelValue = undefined,
  defaultValue = 0,
  step = 1,
  largeStep = 15,
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
   * Native form field name; a hidden input submits the angle in degrees.
   *
   * @default undefined
   */
  readonly name?: string;

  /**
   * Controlled angle in degrees; any number is wrapped into `[0, 360)`.
   *
   * @default undefined
   */
  readonly modelValue?: number;

  /**
   * Initial uncontrolled angle, also restored by form reset.
   *
   * @default 0
   */
  readonly defaultValue?: number;

  /**
   * Angle step in degrees.
   *
   * @default 1
   */
  readonly step?: number;

  /**
   * Angle step for Page Up and Page Down.
   *
   * @default 15
   */
  readonly largeStep?: number;

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
   * Formats `aria-valuetext`.
   *
   * @default (angle) => `${angle} degrees`
   */
  readonly getValueText?: (angle: number) => string;

  /**
   * Accessible name when no label or `aria-labelledby` supplies one.
   *
   * @default undefined
   */
  readonly ariaLabel?: string;

  /**
   * Ids that label the picker.
   *
   * @default undefined
   */
  readonly ariaLabelledby?: string;

  /**
   * Ids that describe the picker.
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
  readonly ariaInvalid?: AnglePickerAriaInvalid;
}>();

const emit = defineEmits<{
  /** Fired for every requested angle, including drag frames. */
  "update:modelValue": [angle: number];

  /** Fired once a change is committed (key press, pointer release, or API). */
  change: [angle: number, source: AnglePickerChangeSource];
}>();

defineSlots<{
  /** Renders the dial and indicator with the current angle. */
  default?(props: AnglePickerSlotState): unknown;
}>();

const element = useTemplateRef<HTMLSpanElement>("element");
const controlId = useDeterministicId({ id: () => id, hint: "angle" });
const stepSize = computed(() => (Number.isFinite(step) && step > 0 && step < 360 ? step : 1));
const largeSize = computed(() => (Number.isFinite(largeStep) && largeStep > 0 ? largeStep : 15));
const state = useControllableState<number>({
  value: () => (modelValue === undefined ? undefined : snapAngle(modelValue, stepSize.value)),
  defaultValue: () => snapAngle(defaultValue, stepSize.value),
  onChange: (value) => emit("update:modelValue", value),
});
const angle = computed(() => snapAngle(state.value.value, stepSize.value));
const maxAngle = computed(() => wrapAngle(360 - stepSize.value));
const dragging = shallowRef(false);
const committed = shallowRef(angle.value);
const ariaInvalidValue = computed(() => {
  if (ariaInvalid === false) return undefined;
  return ariaInvalid === true ? "true" : ariaInvalid;
});
const dataState = computed<AnglePickerState>(() => {
  if (disabled) return "disabled";
  if (readOnly) return "readonly";
  return dragging.value ? "dragging" : "idle";
});
const rootStyle = computed<AnglePickerStyle>(() => ({
  "--vize-angle-picker-angle": `${angle.value}deg`,
}));
const valueText = computed(() =>
  (getValueText ?? ((value: number) => `${value} degrees`))(angle.value),
);
const editable = () => !disabled && !readOnly;

function request(next: number): boolean {
  return state.set(snapAngle(next, stepSize.value));
}

function commit(source: AnglePickerChangeSource): void {
  if (committed.value === angle.value) return;
  committed.value = angle.value;
  emit("change", angle.value, source);
}

function keyTarget(key: string): number | undefined {
  // Arrow keys wrap around the circle; Home/End go to 0° and the last step.
  if (key === "ArrowUp" || key === "ArrowRight") return angle.value + stepSize.value;
  if (key === "ArrowDown" || key === "ArrowLeft") return angle.value - stepSize.value;
  if (key === "PageUp") return angle.value + largeSize.value;
  if (key === "PageDown") return angle.value - largeSize.value;
  if (key === "Home") return 0;
  if (key === "End") return maxAngle.value;
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
    request(pointer);
  },
  onEnd: () => {
    dragging.value = false;
    commit("pointer");
  },
});

watch(
  element,
  (target, _previous, onCleanup) => {
    const owner = target?.closest("form");
    if (owner === null || owner === undefined) return;
    const onReset = () => {
      if (!state.controlled.value) state.reset();
      committed.value = angle.value;
    };
    owner.addEventListener("reset", onReset);
    onCleanup(() => owner.removeEventListener("reset", onReset));
  },
  { flush: "post", immediate: true },
);

const slotState = computed<AnglePickerSlotState>(() => ({
  value: angle.value,
  angle: angle.value,
  percent: (angle.value / 360) * 100,
  min: 0,
  max: maxAngle.value,
  dragging: dragging.value,
  disabled,
  state: dataState.value,
}));

type AnglePickerSetupExpose = Omit<AnglePickerExpose, keyof AnglePickerSlotState | "element"> & {
  readonly [Key in keyof AnglePickerSlotState]: ComputedRef<AnglePickerSlotState[Key]>;
} & { readonly element: typeof element };

function field<Key extends keyof AnglePickerSlotState>(
  key: Key,
): ComputedRef<AnglePickerSlotState[Key]> {
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
    committed.value = angle.value;
    return changed;
  },
} satisfies AnglePickerSetupExpose;

defineExpose(exposed);
</script>

<template>
  <span
    :id="controlId"
    ref="element"
    role="slider"
    tabindex="0"
    :aria-valuenow="angle"
    aria-valuemin="0"
    :aria-valuemax="maxAngle"
    :aria-valuetext="valueText"
    :aria-label="ariaLabel"
    :aria-labelledby="ariaLabelledby"
    :aria-describedby="ariaDescribedby"
    :aria-errormessage="ariaInvalidValue === undefined ? undefined : ariaErrormessage"
    :aria-invalid="ariaInvalidValue"
    :aria-disabled="disabled ? 'true' : undefined"
    :aria-readonly="readOnly ? 'true' : undefined"
    part="root"
    data-vize-ui="angle-picker"
    :data-state="dataState"
    :style="rootStyle"
    @keydown="onKeydown"
    @pointerdown="drag.onPointerdown"
  >
    <input v-if="name !== undefined" type="hidden" :name :value="String(angle)" :disabled />
    <slot v-bind="slotState" />
  </span>
</template>

<style scoped>
/* Headless by design. Rotate an indicator with --vize-angle-picker-angle. */
</style>
