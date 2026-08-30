<script setup lang="ts">
import { computed, nextTick, useTemplateRef, watch, watchEffect } from "vue";
import type { ComputedRef } from "vue";

import { useControllableState } from "../../../controllable-state.ts";
import { useDeterministicId } from "../../../deterministic-id.ts";
import type {
  NativeSelectAriaInvalid,
  NativeSelectDirection,
  NativeSelectEmits,
  NativeSelectExpose,
  NativeSelectOption,
  NativeSelectOptionState,
  NativeSelectProps,
  NativeSelectSelectionMode,
  NativeSelectSlotState,
  NativeSelectState,
  NativeSelectValue,
} from "./native-select-types.ts";
import {
  areNativeSelectValuesEqual,
  nativeSelectSelectedValues,
  normalizeNativeSelectValue,
} from "./native-select-value.ts";

const {
  id = undefined,
  name = undefined,
  modelValue = undefined,
  defaultValue = undefined,
  options = [],
  multiple = false,
  size = undefined,
  disabled = false,
  required = false,
  direction = "ltr",
  ariaLabel = undefined,
  ariaLabelledby = undefined,
  ariaDescribedby = undefined,
  ariaErrormessage = undefined,
  ariaInvalid = false,
} = defineProps<NativeSelectProps>();

defineSlots<{
  /** Renders optional custom option content with the current NativeSelect state. */
  default?(props: NativeSelectSlotState): unknown;
}>();

const emit = defineEmits<NativeSelectEmits>();

const element = useTemplateRef<HTMLSelectElement>("element");
const controlId = useDeterministicId({ id: () => id, hint: "native-select" });
const selectionMode = computed<NativeSelectSelectionMode>(() => (multiple ? "multiple" : "single"));
const valueState = useControllableState<NativeSelectValue>({
  value: () =>
    modelValue === undefined ? undefined : normalizeNativeSelectValue(modelValue, multiple),
  defaultValue: () => normalizeNativeSelectValue(defaultValue, multiple),
  equals: areNativeSelectValuesEqual,
  onChange: (value) => emit("update:modelValue", value),
});
const value = computed(() => normalizeNativeSelectValue(valueState.value.value, multiple));
const selectedValues = computed<readonly string[]>(() => nativeSelectSelectedValues(value.value));
const selectedCount = computed<number>(() => selectedValues.value.length);
const selectedSet = computed(() => new Set(selectedValues.value));
const singleValue = computed(() => (typeof value.value === "string" ? value.value : ""));
const nativeValue = computed(() => (multiple ? undefined : singleValue.value));
const ariaInvalidValue = computed(() => {
  if (ariaInvalid === false) return undefined;
  return ariaInvalid === true ? "true" : ariaInvalid;
});
const invalid = computed(() => ariaInvalidValue.value !== undefined);
const state = computed<NativeSelectState>(() => {
  if (disabled) return "disabled";
  return selectedValues.value.length === 0 ? "empty" : "selected";
});
const dataValue = computed(() =>
  selectedValues.value.length === 1 ? selectedValues.value[0] : undefined,
);
const slotState = computed<NativeSelectSlotState>(() => ({
  direction,
  disabled,
  invalid: invalid.value,
  multiple,
  required,
  selectedValues: selectedValues.value,
  selectionMode: selectionMode.value,
  state: state.value,
  value: value.value,
}));

function isOptionSelected(optionValue: string): boolean {
  return selectedSet.value.has(optionValue);
}

function optionState(option: NativeSelectOption): NativeSelectOptionState {
  if (option.disabled === true) return "disabled";
  return isOptionSelected(option.value) ? "selected" : "unselected";
}

function syncNativeValue(): void {
  if (element.value === null) return;

  if (multiple) {
    syncNativeOptions(element.value);
    return;
  }

  syncNativeSingleValue(element.value);
}

function syncNativeOptions(select: HTMLSelectElement): void {
  for (const option of select.options) option.selected = selectedSet.value.has(option.value);
}

function syncNativeSingleValue(select: HTMLSelectElement): void {
  if (select.value !== singleValue.value) select.value = singleValue.value;
}

watchEffect(syncNativeValue);

watch(
  element,
  (select, _previous, onCleanup) => {
    const form = select?.form;
    if (form === undefined || form === null) return;
    const onReset = () => {
      if (!valueState.controlled.value) valueState.reset();
      void nextTick(syncNativeValue);
    };
    form.addEventListener("reset", onReset);
    onCleanup(() => form.removeEventListener("reset", onReset));
  },
  { flush: "post", immediate: true },
);

function readSelectValue(select: HTMLSelectElement): NativeSelectValue {
  if (!multiple) return select.value;
  const values: string[] = [];
  for (const option of select.selectedOptions) values.push(option.value);
  return values;
}

function commitValue(next: NativeSelectValue, nativeEvent: Event | null): boolean {
  const normalizedNext = normalizeNativeSelectValue(next, multiple);
  let previous: NativeSelectValue = multiple ? [] : "";
  const changed = valueState.set((current) => {
    previous = normalizeNativeSelectValue(current, multiple);
    return normalizedNext;
  });
  if (changed && nativeEvent !== null) emit("change", normalizedNext, previous, nativeEvent);
  void nextTick(syncNativeValue);
  return changed;
}

function onChange(event: Event): void {
  if (!(event.currentTarget instanceof HTMLSelectElement)) return;
  commitValue(readSelectValue(event.currentTarget), event);
}

function focus(options?: FocusOptions): void {
  element.value?.focus(options);
}

function setValue(next: NativeSelectValue): boolean {
  return commitValue(next, null);
}

function clear(): boolean {
  return commitValue(multiple ? [] : "", null);
}

type NativeSelectSetupExpose = Omit<
  NativeSelectExpose,
  | "direction"
  | "disabled"
  | "element"
  | "id"
  | "invalid"
  | "multiple"
  | "required"
  | "selectedValues"
  | "selectionMode"
  | "state"
  | "value"
> & {
  readonly direction: ComputedRef<NativeSelectDirection>;
  readonly disabled: ComputedRef<boolean>;
  readonly element: typeof element;
  readonly id: ComputedRef<string>;
  readonly invalid: ComputedRef<boolean>;
  readonly multiple: ComputedRef<boolean>;
  readonly required: ComputedRef<boolean>;
  readonly selectedValues: ComputedRef<readonly string[]>;
  readonly selectionMode: ComputedRef<NativeSelectSelectionMode>;
  readonly state: ComputedRef<NativeSelectState>;
  readonly value: ComputedRef<NativeSelectValue>;
};

const exposed = {
  clear,
  direction: computed(() => direction),
  disabled: computed(() => disabled),
  element,
  focus,
  id: controlId,
  invalid,
  multiple: computed(() => multiple),
  required: computed(() => required),
  reset: valueState.reset,
  selectedValues,
  selectionMode,
  setValue,
  state,
  value,
} satisfies NativeSelectSetupExpose;

defineExpose(exposed);
</script>

<template>
  <select
    :id="controlId"
    ref="element"
    :name
    :value="nativeValue"
    :multiple
    :size
    :disabled
    :required
    :dir="direction"
    :aria-label="ariaLabel"
    :aria-labelledby="ariaLabelledby"
    :aria-describedby="ariaDescribedby"
    :aria-errormessage="ariaInvalidValue === undefined ? undefined : ariaErrormessage"
    :aria-invalid="ariaInvalidValue"
    data-vize-ui="native-select"
    part="root"
    :data-state="state"
    :data-disabled="disabled ? 'true' : undefined"
    :data-required="required ? 'true' : undefined"
    :data-invalid="invalid ? 'true' : undefined"
    :data-selection-mode="selectionMode"
    :data-selection-count="selectedCount"
    :data-direction="direction"
    :data-value="dataValue"
    @change="onChange"
  >
    <option
      v-for="option in options"
      :key="option.value"
      :value="option.value"
      :disabled="option.disabled === true"
      :selected="isOptionSelected(option.value)"
      data-vize-ui="native-select-option"
      part="option"
      :data-state="optionState(option)"
      :data-value="option.value"
      :data-selected="isOptionSelected(option.value) ? 'true' : undefined"
      :data-disabled="option.disabled === true ? 'true' : undefined"
    >
      {{ option.label }}
    </option>
    <slot v-bind="slotState" />
  </select>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
