<script setup lang="ts">
import { computed, useTemplateRef, watch } from "vue";
import type { ComputedRef } from "vue";

import { useControllableState } from "./controllable-state.ts";
import { useDeterministicId } from "./deterministic-id.ts";
import { radioGroupContext } from "./radio-group-context.ts";
import type {
  RadioGroupAriaInvalid,
  RadioGroupExpose,
  RadioGroupOrientation,
  RadioGroupSlotState,
  RadioGroupState,
  RadioGroupValue,
} from "./radio-group-types.ts";

const {
  id = undefined,
  name = undefined,
  modelValue = undefined,
  defaultValue = null,
  disabled = false,
  required = false,
  orientation = "vertical",
  ariaLabel = undefined,
  ariaLabelledby = undefined,
  ariaDescribedby = undefined,
  ariaErrormessage = undefined,
  ariaInvalid = false,
} = defineProps<{
  /**
   * Consumer-owned radio group id. `null` and `undefined` select a deterministic fallback.
   *
   * @default undefined
   */
  readonly id?: string | null;

  /**
   * Native radio name shared by every item for form submission and browser grouping.
   *
   * @default undefined
   */
  readonly name?: string;

  /**
   * Controlled selected value. `undefined` selects uncontrolled behavior; `null` clears selection.
   *
   * @default undefined
   */
  readonly modelValue?: RadioGroupValue;

  /**
   * Initial value for uncontrolled use and the value restored by form reset.
   *
   * @default null
   */
  readonly defaultValue?: RadioGroupValue;

  /**
   * Disable every radio item and native form submission.
   *
   * @default false
   */
  readonly disabled?: boolean;

  /**
   * Mark the native radio set as required for constraint validation.
   *
   * @default false
   */
  readonly required?: boolean;

  /**
   * Directional layout hint exposed for consumer-owned styles.
   *
   * @default "vertical"
   */
  readonly orientation?: RadioGroupOrientation;

  /**
   * Accessible name when no visible label or `aria-labelledby` supplies one.
   *
   * @default undefined
   */
  readonly ariaLabel?: string;

  /**
   * Space-separated ids that label the radio group.
   *
   * @default undefined
   */
  readonly ariaLabelledby?: string;

  /**
   * Space-separated ids that describe the radio group.
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
  readonly ariaInvalid?: RadioGroupAriaInvalid;
}>();

defineSlots<{
  /** Compound RadioGroup items. Receives current value, validity, and availability state. */
  default(props: RadioGroupSlotState): unknown;
}>();

const emit = defineEmits<{
  /** Fired when the selected value requests a new controlled value. */
  "update:modelValue": [value: RadioGroupValue];

  /** Fired after user selection requests a distinct radio value. */
  change: [value: string, previous: RadioGroupValue, nativeEvent: Event];
}>();

const element = useTemplateRef<HTMLDivElement>("element");
const groupId = useDeterministicId({ id: () => id, hint: "radio-group" });
const valueState = useControllableState<RadioGroupValue>({
  value: () => modelValue,
  defaultValue: () => defaultValue,
  onChange: (value) => emit("update:modelValue", value),
});
const selectedValue = valueState.value;
const disabledState = computed(() => disabled);
const requiredState = computed(() => required);
const orientationState = computed(() => orientation);
const ariaInvalidValue = computed(() => {
  if (ariaInvalid === false) return undefined;
  return ariaInvalid === true ? "true" : ariaInvalid;
});
const invalid = computed(() => ariaInvalidValue.value !== undefined);
const state = computed<RadioGroupState>(() => {
  if (disabledState.value) return "disabled";
  return selectedValue.value === null ? "empty" : "selected";
});
const slotState = computed<RadioGroupSlotState>(() => ({
  disabled: disabledState.value,
  invalid: invalid.value,
  orientation: orientationState.value,
  required: requiredState.value,
  state: state.value,
  value: selectedValue.value,
}));

watch(
  element,
  (root, _previous, onCleanup) => {
    const form = root?.closest("form");
    if (form === undefined || form === null) return;
    const onReset = () => {
      if (!valueState.controlled.value) valueState.reset();
    };
    form.addEventListener("reset", onReset);
    onCleanup(() => form.removeEventListener("reset", onReset));
  },
  { flush: "post", immediate: true },
);

function selectValue(next: string, nativeEvent: Event): boolean {
  const previous = selectedValue.value;
  const changed = valueState.set(next);
  if (changed) emit("change", next, previous, nativeEvent);
  return changed;
}

function setValue(next: RadioGroupValue): boolean {
  return valueState.set(next);
}

function focus(options?: FocusOptions): void {
  const checked = element.value?.querySelector<HTMLInputElement>(
    'input[data-vize-ui="radio-group-item"]:checked:not(:disabled)',
  );
  const first = element.value?.querySelector<HTMLInputElement>(
    'input[data-vize-ui="radio-group-item"]:not(:disabled)',
  );
  (checked ?? first)?.focus(options);
}

const context = radioGroupContext.provide({
  disabled: disabledState,
  id: groupId,
  invalid,
  name: computed(() => name),
  orientation: orientationState,
  required: requiredState,
  selectValue,
  state,
  value: selectedValue,
});

type RadioGroupSetupExpose = Omit<
  RadioGroupExpose,
  "disabled" | "id" | "invalid" | "orientation" | "required" | "state" | "value"
> & {
  readonly disabled: ComputedRef<boolean>;
  readonly element: typeof element;
  readonly id: ComputedRef<string>;
  readonly invalid: ComputedRef<boolean>;
  readonly orientation: ComputedRef<RadioGroupOrientation>;
  readonly required: ComputedRef<boolean>;
  readonly state: ComputedRef<RadioGroupState>;
  readonly value: ComputedRef<RadioGroupValue>;
};

const exposed = {
  disabled: disabledState,
  element,
  focus,
  id: groupId,
  invalid,
  orientation: orientationState,
  required: requiredState,
  reset: valueState.reset,
  setValue,
  state,
  value: selectedValue,
} satisfies RadioGroupSetupExpose;

defineExpose(exposed);
</script>

<template>
  <div
    :id="context.id.value"
    ref="element"
    role="radiogroup"
    :aria-label="ariaLabel"
    :aria-labelledby="ariaLabelledby"
    :aria-describedby="ariaDescribedby"
    :aria-orientation="orientationState"
    :aria-errormessage="ariaInvalidValue === undefined ? undefined : ariaErrormessage"
    :aria-invalid="ariaInvalidValue"
    :aria-disabled="disabledState ? 'true' : undefined"
    :aria-required="requiredState ? 'true' : undefined"
    data-vize-ui="radio-group"
    part="root"
    :data-state="state"
    :data-disabled="disabledState ? 'true' : undefined"
    :data-required="requiredState ? 'true' : undefined"
    :data-invalid="invalid ? 'true' : undefined"
    :data-orientation="orientationState"
    :data-value="selectedValue ?? undefined"
  >
    <slot v-bind="slotState" />
  </div>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
