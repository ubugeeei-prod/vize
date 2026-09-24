<script setup lang="ts">
import { computed, ref, shallowRef, useTemplateRef, watch } from "vue";
import type { ComputedRef } from "vue";

import { useControllableState } from "../../foundations/controllable-state/controllable-state.ts";
import { useDeterministicId } from "../../foundations/id/deterministic-id.ts";
import { useLocale } from "../../i18n/locale/locale.ts";
import { numberFieldContext } from "./number-field-context.ts";
import { createNumberFieldParser } from "./number-field-parser.ts";
import {
  NUMBER_FIELD_DEFAULT_PERCENT_STEP,
  NUMBER_FIELD_DEFAULT_STEP,
  canStepNumberFieldValue,
  clampNumberFieldValue,
  getNumberFieldState,
  normalizeNumberFieldBounds,
  snapNumberFieldValue,
  stepNumberFieldValue,
} from "./number-field-state.ts";
import type {
  NumberFieldChangeSource,
  NumberFieldEmits,
  NumberFieldExpose,
  NumberFieldProps,
  NumberFieldSlotState,
  NumberFieldStepDirection,
  NumberFieldValue,
} from "./number-field-types.ts";

const {
  id = undefined,
  name = undefined,
  form = undefined,
  modelValue = undefined,
  defaultValue = null,
  min = undefined,
  max = undefined,
  step = undefined,
  largeStep = undefined,
  locale = undefined,
  formatOptions = undefined,
  clampOnCommit = true,
  snapOnCommit = false,
  allowWheel = false,
  holdDelay = 400,
  holdInterval = 60,
  disabled = false,
  readOnly = false,
  required = false,
  ariaLabel = undefined,
  ariaLabelledby = undefined,
  ariaDescribedby = undefined,
  ariaErrormessage = undefined,
  ariaInvalid = false,
} = defineProps<NumberFieldProps>();

const emit = defineEmits<NumberFieldEmits>();

defineSlots<{
  /** Renders the input, triggers, and any adornments with normalized state. */
  default(props: NumberFieldSlotState): unknown;
}>();

const root = useTemplateRef<HTMLDivElement>("root");
const inputElement = shallowRef<HTMLInputElement | null>(null);
const inputId = useDeterministicId({ id: () => id, hint: "number-field" });
const contextLocale = useLocale();
const resolvedLocale = computed(() => locale ?? contextLocale.value);
const parser = computed(() => createNumberFieldParser(resolvedLocale.value, formatOptions));
const bounds = computed(() =>
  normalizeNumberFieldBounds({
    min,
    max,
    step,
    largeStep,
    defaultStep:
      formatOptions?.style === "percent"
        ? NUMBER_FIELD_DEFAULT_PERCENT_STEP
        : NUMBER_FIELD_DEFAULT_STEP,
  }),
);

function finiteOrNull(value: NumberFieldValue | undefined): NumberFieldValue {
  return typeof value === "number" && Number.isFinite(value) ? value : null;
}

const state = useControllableState<NumberFieldValue>({
  value: () => (modelValue === undefined ? undefined : finiteOrNull(modelValue)),
  defaultValue: () => finiteOrNull(defaultValue),
  onChange: (value) => emit("update:modelValue", value),
});
const value = state.value;
const formattedValue = computed(() =>
  value.value === null ? "" : parser.value.format(value.value),
);
// Uncommitted typing lives in `draft`; otherwise the input mirrors the committed value.
const draft = ref<string | null>(null);
const inputText = computed(() => draft.value ?? formattedValue.value);
const ariaInvalidValue = computed(() => {
  if (ariaInvalid === false) return undefined;
  return ariaInvalid === true ? "true" : ariaInvalid;
});
const invalid = computed(() => ariaInvalidValue.value !== undefined);
const disabledState = computed(() => disabled);
const readOnlyState = computed(() => readOnly);
const requiredState = computed(() => required);
const dataState = computed(() =>
  getNumberFieldState({
    value: value.value,
    bounds: bounds.value,
    disabled,
    readOnly,
    invalid: invalid.value,
  }),
);

function isEditable(): boolean {
  return !disabled && !readOnly;
}

function normalizeCommitted(next: NumberFieldValue): NumberFieldValue {
  if (next === null) return null;
  let normalized = next;
  if (clampOnCommit) normalized = clampNumberFieldValue(normalized, bounds.value);
  if (snapOnCommit) normalized = snapNumberFieldValue(normalized, bounds.value);
  return normalized;
}

function apply(next: NumberFieldValue, source: NumberFieldChangeSource): boolean {
  const previous = value.value;
  const changed = state.set(next);
  // Dropping the draft re-formats equivalent text ("1.0" -> "1") and shows the
  // controlled value again when a parent rejects the request.
  draft.value = null;
  if (changed) emit("change", next, previous, source);
  return changed;
}

function commit(source: NumberFieldChangeSource): boolean {
  const parsed = parser.value.parse(inputText.value);
  if (parsed !== null && Number.isNaN(parsed)) {
    draft.value = null;
    return false;
  }
  return apply(normalizeCommitted(parsed), source);
}

function directionSign(direction: NumberFieldStepDirection): 1 | -1 {
  return direction === "increment" ? 1 : -1;
}

function canStep(direction: NumberFieldStepDirection): boolean {
  return (
    isEditable() && canStepNumberFieldValue(value.value, directionSign(direction), bounds.value)
  );
}

function currentForStepping(): NumberFieldValue {
  // Stepping starts from the typed text when it parses, so "12" then ArrowUp yields 13.
  const parsed = parser.value.parse(inputText.value);
  if (parsed === null) return null;
  return Number.isNaN(parsed) ? value.value : parsed;
}

function stepBy(
  direction: NumberFieldStepDirection,
  amount: number,
  source: NumberFieldChangeSource,
): boolean {
  if (!isEditable()) return false;
  const next = stepNumberFieldValue(
    currentForStepping(),
    directionSign(direction),
    bounds.value,
    amount,
  );
  return apply(next, source);
}

function stepContext(
  direction: NumberFieldStepDirection,
  size: "step" | "largeStep",
  source: NumberFieldChangeSource,
): boolean {
  return stepBy(direction, bounds.value[size], source);
}

function stepToBound(bound: "min" | "max", source: NumberFieldChangeSource): boolean {
  if (!isEditable()) return false;
  const target = bounds.value[bound];
  if (!Number.isFinite(target)) return false;
  return apply(target, source);
}

watch(
  inputElement,
  (input, _previous, onCleanup) => {
    const owner = input?.form;
    if (owner === undefined || owner === null) return;
    const onReset = () => {
      if (!state.controlled.value) state.reset();
      draft.value = null;
    };
    owner.addEventListener("reset", onReset);
    onCleanup(() => owner.removeEventListener("reset", onReset));
  },
  { flush: "post" },
);

const slotState = computed<NumberFieldSlotState>(() => ({
  value: value.value,
  formattedValue: formattedValue.value,
  inputText: inputText.value,
  min: bounds.value.min,
  max: bounds.value.max,
  step: bounds.value.step,
  largeStep: bounds.value.largeStep,
  locale: parser.value.locale,
  canIncrement: canStep("increment"),
  canDecrement: canStep("decrement"),
  disabled,
  readOnly,
  required,
  invalid: invalid.value,
  state: dataState.value,
}));

numberFieldContext.provide({
  inputId,
  form: computed(() => form),
  value,
  inputText,
  setDraft: (text: string) => {
    draft.value = text;
  },
  bounds,
  parser,
  state: dataState,
  disabled: disabledState,
  readOnly: readOnlyState,
  required: requiredState,
  invalid,
  allowWheel: computed(() => allowWheel),
  holdDelay: computed(() => (Number.isFinite(holdDelay) && holdDelay >= 0 ? holdDelay : 400)),
  holdInterval: computed(() =>
    Number.isFinite(holdInterval) && holdInterval > 0 ? holdInterval : 60,
  ),
  ariaInvalid: ariaInvalidValue,
  ariaLabel: computed(() => ariaLabel),
  ariaLabelledby: computed(() => ariaLabelledby),
  ariaDescribedby: computed(() => ariaDescribedby),
  ariaErrormessage: computed(() => ariaErrormessage),
  inputElement,
  canStep,
  step: stepContext,
  stepToBound,
  commit,
});

function focus(options?: FocusOptions): void {
  inputElement.value?.focus(options);
}

function stepCount(count: number | undefined): number {
  return typeof count === "number" && Number.isFinite(count) && count > 0 ? count : 1;
}

type NumberFieldSetupExpose = Omit<
  NumberFieldExpose,
  keyof NumberFieldSlotState | "input" | "root"
> & {
  readonly [Key in keyof NumberFieldSlotState]: ComputedRef<NumberFieldSlotState[Key]>;
} & {
  readonly input: typeof inputElement;
  readonly root: typeof root;
};

function field<Key extends keyof NumberFieldSlotState>(
  key: Key,
): ComputedRef<NumberFieldSlotState[Key]> {
  return computed(() => slotState.value[key]);
}

const exposed = {
  value: field("value"),
  formattedValue: field("formattedValue"),
  inputText: field("inputText"),
  min: field("min"),
  max: field("max"),
  step: field("step"),
  largeStep: field("largeStep"),
  locale: field("locale"),
  canIncrement: field("canIncrement"),
  canDecrement: field("canDecrement"),
  disabled: field("disabled"),
  readOnly: field("readOnly"),
  required: field("required"),
  invalid: field("invalid"),
  state: field("state"),
  root,
  input: inputElement,
  focus,
  setValue: (next: NumberFieldValue) => apply(normalizeCommitted(finiteOrNull(next)), "api"),
  increment: (count?: number) => stepBy("increment", bounds.value.step * stepCount(count), "api"),
  decrement: (count?: number) => stepBy("decrement", bounds.value.step * stepCount(count), "api"),
  commit: () => commit("api"),
  reset: () => {
    const changed = state.reset();
    draft.value = null;
    return changed;
  },
} satisfies NumberFieldSetupExpose;

defineExpose(exposed);
</script>

<template>
  <div
    ref="root"
    part="root"
    data-vize-ui="number-field"
    :data-state="dataState"
    :data-disabled="disabled ? 'true' : undefined"
    :data-readonly="readOnly ? 'true' : undefined"
    :data-required="required ? 'true' : undefined"
    :data-invalid="invalid ? 'true' : undefined"
    :data-empty="value === null ? 'true' : undefined"
  >
    <input
      v-if="name !== undefined"
      type="hidden"
      :name
      :form
      :value="value === null ? '' : String(value)"
      :disabled
      data-vize-ui="number-field-value"
    />
    <slot v-bind="slotState" />
  </div>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
