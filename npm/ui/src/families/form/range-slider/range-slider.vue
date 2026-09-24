<script setup lang="ts">
import { computed, onScopeDispose, shallowRef, useTemplateRef, watch } from "vue";
import type { ComputedRef } from "vue";

// The context module is imported first so shared chunks keep the root bundle module order.
import { rangeSliderContext } from "./range-slider-context.ts";
import { useControllableState } from "../../foundations/controllable-state/controllable-state.ts";
import {
  closestRangeSliderThumb,
  normalizeRangeSliderBounds,
  normalizeRangeSliderValue,
  rangeSliderPercent,
  rangeSliderValuesEqual,
  setRangeSliderThumb,
} from "./range-slider-state.ts";
import type {
  RangeSliderChangeSource,
  RangeSliderEmits,
  RangeSliderExpose,
  RangeSliderProps,
  RangeSliderSlotState,
  RangeSliderState,
  RangeSliderStyle,
  RangeSliderValue,
} from "./range-slider-types.ts";

const {
  name = undefined,
  form = undefined,
  modelValue = undefined,
  defaultValue = undefined,
  min = undefined,
  max = undefined,
  step = undefined,
  largeStep = undefined,
  minStepsBetweenThumbs = 0,
  orientation = "horizontal",
  dir = "ltr",
  disabled = false,
  getValueText = undefined,
  ariaLabel = undefined,
  ariaLabelledby = undefined,
  ariaDescribedby = undefined,
  ariaErrormessage = undefined,
  ariaInvalid = false,
} = defineProps<RangeSliderProps>();

const emit = defineEmits<RangeSliderEmits>();

defineSlots<{
  /** Renders the track, range, and one thumb per value with normalized state. */
  default(props: RangeSliderSlotState): unknown;
}>();

const root = useTemplateRef<HTMLSpanElement>("root");
const bounds = computed(() =>
  normalizeRangeSliderBounds({ min, max, step, largeStep, minStepsBetweenThumbs }),
);
const state = useControllableState<RangeSliderValue>({
  value: () =>
    modelValue === undefined ? undefined : normalizeRangeSliderValue(modelValue, bounds.value),
  defaultValue: () => normalizeRangeSliderValue(defaultValue, bounds.value),
  equals: rangeSliderValuesEqual,
  onChange: (value) => emit("update:modelValue", value),
});
const values = state.value;
const activeThumb = shallowRef<number | null>(null);
const thumbs = new Map<number, HTMLElement>();
let committed: RangeSliderValue = values.value;
const ariaInvalidValue = computed(() => {
  if (ariaInvalid === false) return undefined;
  return ariaInvalid === true ? "true" : ariaInvalid;
});
const invalid = computed(() => ariaInvalidValue.value !== undefined);
const percents = computed(() =>
  values.value.map((value) => rangeSliderPercent(value, bounds.value)),
);
const dataState = computed<RangeSliderState>(() => {
  if (disabled) return "disabled";
  if (activeThumb.value !== null) return "dragging";
  return invalid.value ? "invalid" : "idle";
});
const rootStyle = computed<RangeSliderStyle>(() => {
  const start = percents.value.length > 1 ? (percents.value[0] ?? 0) : 0;
  const end = percents.value.at(-1) ?? 0;
  return {
    "--vize-range-slider-range-start": `${start}%`,
    "--vize-range-slider-range-end": `${end}%`,
  };
});
// Thumbs are positional, so each hidden form value is keyed by its thumb slot.
const hiddenValues = computed(() =>
  values.value.map((value, index) => ({ key: `thumb-${index}`, value: String(value) })),
);
const slotState = computed<RangeSliderSlotState>(() => ({
  values: values.value,
  percents: percents.value,
  min: bounds.value.min,
  max: bounds.value.max,
  step: bounds.value.step,
  largeStep: bounds.value.largeStep,
  minDistance: bounds.value.minDistance,
  orientation,
  direction: dir,
  disabled,
  invalid: invalid.value,
  activeThumb: activeThumb.value,
  state: dataState.value,
}));

function setThumb(index: number, value: number): boolean {
  if (disabled) return false;
  return state.set(setRangeSliderThumb(values.value, index, value, bounds.value));
}

function commit(index: number, source: RangeSliderChangeSource): void {
  if (rangeSliderValuesEqual(committed, values.value)) return;
  committed = values.value;
  emit("change", values.value, index, source);
}

function focusThumb(index: number, options?: FocusOptions): void {
  thumbs.get(index)?.focus(options);
}

function registerThumb(index: number, element: HTMLElement | null): void {
  if (element === null) thumbs.delete(index);
  else thumbs.set(index, element);
}

function valueFromPointer(event: PointerEvent, track: HTMLElement): number {
  const rect = track.getBoundingClientRect();
  const vertical = orientation === "vertical";
  const size = vertical ? rect.height : rect.width;
  if (size <= 0) return bounds.value.min;
  let ratio = vertical ? (rect.bottom - event.clientY) / size : (event.clientX - rect.left) / size;
  if (!vertical && dir === "rtl") ratio = 1 - ratio;
  const clamped = Math.min(Math.max(ratio, 0), 1);
  return bounds.value.min + clamped * (bounds.value.max - bounds.value.min);
}

let stopDrag: (() => void) | undefined;

function startDrag(event: PointerEvent, track: HTMLElement): void {
  if (disabled || event.button !== 0) return;
  event.preventDefault();
  stopDrag?.();
  const index = closestRangeSliderThumb(values.value, valueFromPointer(event, track));
  activeThumb.value = index;
  setThumb(index, valueFromPointer(event, track));
  focusThumb(index);
  const pointerId = event.pointerId;
  try {
    track.setPointerCapture(pointerId);
  } catch {
    // Synthetic or already-released pointers cannot be captured; listeners still work.
  }
  const onMove = (move: PointerEvent) => {
    if (move.pointerId !== pointerId) return;
    setThumb(index, valueFromPointer(move, track));
  };
  const onEnd = (end: PointerEvent) => {
    if (end.pointerId !== pointerId) return;
    stopDrag?.();
    commit(index, "pointer");
  };
  track.addEventListener("pointermove", onMove);
  track.addEventListener("pointerup", onEnd);
  track.addEventListener("pointercancel", onEnd);
  stopDrag = () => {
    track.removeEventListener("pointermove", onMove);
    track.removeEventListener("pointerup", onEnd);
    track.removeEventListener("pointercancel", onEnd);
    activeThumb.value = null;
    stopDrag = undefined;
  };
}

onScopeDispose(() => stopDrag?.());

watch(
  root,
  (element, _previous, onCleanup) => {
    const owner =
      form === undefined ? element?.closest("form") : element?.ownerDocument.getElementById(form);
    if (!(owner instanceof HTMLFormElement)) return;
    const onReset = () => {
      if (!state.controlled.value) state.reset();
      committed = values.value;
    };
    owner.addEventListener("reset", onReset);
    onCleanup(() => owner.removeEventListener("reset", onReset));
  },
  { flush: "post", immediate: true },
);

rangeSliderContext.provide({
  values,
  bounds,
  orientation: computed(() => orientation),
  direction: computed(() => dir),
  disabled: computed(() => disabled),
  activeThumb,
  getValueText: computed(() => getValueText),
  ariaLabelledby: computed(() => ariaLabelledby),
  ariaDescribedby: computed(() => ariaDescribedby),
  ariaErrormessage: computed(() => ariaErrormessage),
  ariaInvalid: ariaInvalidValue,
  registerThumb,
  focusThumb,
  setThumb,
  commit,
  startDrag,
});

type RangeSliderSetupExpose = Omit<RangeSliderExpose, keyof RangeSliderSlotState | "root"> & {
  readonly [Key in keyof RangeSliderSlotState]: ComputedRef<RangeSliderSlotState[Key]>;
} & { readonly root: typeof root };

function field<Key extends keyof RangeSliderSlotState>(
  key: Key,
): ComputedRef<RangeSliderSlotState[Key]> {
  return computed(() => slotState.value[key]);
}

const exposed = {
  values: field("values"),
  percents: field("percents"),
  min: field("min"),
  max: field("max"),
  step: field("step"),
  largeStep: field("largeStep"),
  minDistance: field("minDistance"),
  orientation: field("orientation"),
  direction: field("direction"),
  disabled: field("disabled"),
  invalid: field("invalid"),
  activeThumb: field("activeThumb"),
  state: field("state"),
  root,
  focusThumb,
  setThumbValue: (index: number, value: number) => {
    const changed = setThumb(index, value);
    commit(index, "api");
    return changed;
  },
  setValue: (next: RangeSliderValue) => {
    const changed = state.set(normalizeRangeSliderValue(next, bounds.value));
    commit(0, "api");
    return changed;
  },
  reset: () => {
    const changed = state.reset();
    committed = values.value;
    return changed;
  },
} satisfies RangeSliderSetupExpose;

defineExpose(exposed);
</script>

<template>
  <span
    ref="root"
    role="group"
    :aria-label="ariaLabel"
    :aria-labelledby="ariaLabelledby"
    part="root"
    data-vize-ui="range-slider"
    :data-state="dataState"
    :data-orientation="orientation"
    :data-dir="dir"
    :data-disabled="disabled ? 'true' : undefined"
    :data-invalid="invalid ? 'true' : undefined"
    :style="rootStyle"
  >
    <template v-if="name !== undefined">
      <input
        v-for="entry in hiddenValues"
        :key="entry.key"
        type="hidden"
        :name
        :form
        :value="entry.value"
        :disabled
        data-vize-ui="range-slider-value"
      />
    </template>
    <slot v-bind="slotState" />
  </span>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
