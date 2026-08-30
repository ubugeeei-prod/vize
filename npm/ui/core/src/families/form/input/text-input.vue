<script setup lang="ts">
import { computed, nextTick, ref, useTemplateRef, watch, watchEffect } from "vue";

import { useControllableState } from "../../../controllable-state.ts";
import { useDeterministicId } from "../../foundations/id/deterministic-id.ts";
import type {
  InputAriaInvalid,
  InputEnterKeyHint,
  InputExpose,
  InputInputMode,
  InputType,
} from "./input-types.ts";

const {
  id = undefined,
  name = undefined,
  type = "text",
  modelValue = undefined,
  defaultValue = "",
  disabled = false,
  readOnly = false,
  required = false,
  placeholder = undefined,
  autocomplete = undefined,
  inputMode = undefined,
  enterKeyHint = undefined,
  ariaLabel = undefined,
  ariaLabelledby = undefined,
  ariaDescribedby = undefined,
  ariaErrormessage = undefined,
  ariaInvalid = false,
} = defineProps<{
  /**
   * Consumer-owned control id. `null` and `undefined` select a deterministic fallback.
   *
   * @default undefined
   */
  readonly id?: string | null;

  /**
   * Native form field name.
   *
   * @default undefined
   */
  readonly name?: string;

  /**
   * Text-like native input type.
   *
   * @default "text"
   */
  readonly type?: InputType;

  /**
   * Controlled string value. `undefined` selects uncontrolled behavior.
   *
   * @default undefined
   */
  readonly modelValue?: string;

  /**
   * Initial value for uncontrolled use and the value restored by form reset.
   *
   * @default ""
   */
  readonly defaultValue?: string;

  /**
   * Disable editing, focus, and native form submission.
   *
   * @default false
   */
  readonly disabled?: boolean;

  /**
   * Keep the input focusable while preventing user editing.
   *
   * @default false
   */
  readonly readOnly?: boolean;

  /**
   * Mark the input as required for native constraint validation.
   *
   * @default false
   */
  readonly required?: boolean;

  /**
   * Native placeholder text.
   *
   * @default undefined
   */
  readonly placeholder?: string;

  /**
   * Native autocomplete hint.
   *
   * @default undefined
   */
  readonly autocomplete?: string;

  /**
   * Native virtual-keyboard input mode.
   *
   * @default undefined
   */
  readonly inputMode?: InputInputMode;

  /**
   * Native virtual-keyboard enter key hint.
   *
   * @default undefined
   */
  readonly enterKeyHint?: InputEnterKeyHint;

  /**
   * Accessible name when no label or `aria-labelledby` supplies one.
   *
   * @default undefined
   */
  readonly ariaLabel?: string;

  /**
   * Space-separated ids that label the input.
   *
   * @default undefined
   */
  readonly ariaLabelledby?: string;

  /**
   * Space-separated ids that describe the input.
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
  readonly ariaInvalid?: InputAriaInvalid;
}>();

const emit = defineEmits<{
  /** Fired when the value requests a new controlled string. */
  "update:modelValue": [value: string];

  /** Fired after a native input event with the next string and native `Event`. */
  input: [value: string, nativeEvent: Event];

  /** Fired after native change/commit with the current string and native `Event`. */
  change: [value: string, nativeEvent: Event];

  /** Fired when IME composition starts. */
  compositionStart: [value: string, nativeEvent: CompositionEvent];

  /** Fired when IME composition ends. */
  compositionEnd: [value: string, nativeEvent: CompositionEvent];
}>();

const element = useTemplateRef<HTMLInputElement>("element");
const controlId = useDeterministicId({ id: () => id, hint: "input" });
const composing = ref(false);
let compositionInputValue: string | undefined;
const state = useControllableState({
  value: () => modelValue,
  defaultValue: () => defaultValue,
  onChange: (value) => emit("update:modelValue", value),
});
const value = state.value;
const ariaInvalidValue = computed(() => {
  if (ariaInvalid === false) return undefined;
  return ariaInvalid === true ? "true" : ariaInvalid;
});
const dataState = computed(() => {
  if (disabled) return "disabled";
  return readOnly ? "readonly" : "editable";
});
const dataEmpty = computed(() => (value.value.length === 0 ? "true" : "false"));
const renderedValue = computed(() =>
  composing.value ? (element.value?.value ?? value.value) : value.value,
);

function syncNativeValue(): void {
  if (composing.value) return;
  if (element.value === null) return;
  if (element.value.value !== value.value) element.value.value = value.value;
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

function readInputValue(event: Event): string | undefined {
  if (!(event.currentTarget instanceof HTMLInputElement)) return undefined;
  return event.currentTarget.value;
}

function onInput(event: Event): void {
  const next = readInputValue(event);
  if (next === undefined) return;
  if (composing.value) compositionInputValue = next;
  state.set(next);
  emit("input", next, event);
  void nextTick(syncNativeValue);
}

function onChange(event: Event): void {
  const next = readInputValue(event);
  if (next === undefined) return;
  state.set(next);
  emit("change", next, event);
  void nextTick(syncNativeValue);
}

function onCompositionStart(event: CompositionEvent): void {
  composing.value = true;
  compositionInputValue = undefined;
  emit("compositionStart", value.value, event);
}

function onCompositionEnd(event: CompositionEvent): void {
  composing.value = false;
  const next = readInputValue(event);
  if (next !== undefined && next !== compositionInputValue) state.set(next);
  compositionInputValue = undefined;
  emit("compositionEnd", next ?? value.value, event);
  void nextTick(syncNativeValue);
}

/** Move focus to the native input. */
function focus(options?: FocusOptions): void {
  element.value?.focus(options);
}

/** Select the current native input text. */
function select(): void {
  element.value?.select();
}

function setValue(next: string): boolean {
  const changed = state.set(next);
  void nextTick(syncNativeValue);
  return changed;
}

type InputSetupExpose = Omit<InputExpose, "composing" | "value"> & {
  readonly composing: typeof composing;
  readonly element: typeof element;
  readonly value: typeof value;
};

const exposed = {
  composing,
  element,
  focus,
  reset: state.reset,
  select,
  setValue,
  value,
} satisfies InputSetupExpose;

defineExpose(exposed);
</script>

<template>
  <input
    :id="controlId"
    ref="element"
    :name
    :type
    :value="renderedValue"
    :disabled
    :readonly="readOnly"
    :required
    :placeholder
    :autocomplete
    :inputmode="inputMode"
    :enterkeyhint="enterKeyHint"
    :aria-label="ariaLabel"
    :aria-labelledby="ariaLabelledby"
    :aria-describedby="ariaDescribedby"
    :aria-errormessage="ariaInvalidValue === undefined ? undefined : ariaErrormessage"
    :aria-invalid="ariaInvalidValue"
    data-vize-ui="input"
    :data-state="dataState"
    :data-empty="dataEmpty"
    :data-composing="composing ? 'true' : 'false'"
    @input="onInput"
    @change="onChange"
    @compositionstart="onCompositionStart"
    @compositionend="onCompositionEnd"
  />
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
