<script setup lang="ts">
import { computed, nextTick, ref, useTemplateRef, watch, watchEffect } from "vue";

import { useControllableState } from "../../../controllable-state.ts";
import { useDeterministicId } from "../../../deterministic-id.ts";
import type { TextareaAriaInvalid, TextareaExpose, TextareaWrap } from "./textarea-types.ts";

const {
  id = undefined,
  name = undefined,
  modelValue = undefined,
  defaultValue = "",
  disabled = false,
  readOnly = false,
  required = false,
  placeholder = undefined,
  autocomplete = undefined,
  rows = undefined,
  cols = undefined,
  minlength = undefined,
  maxlength = undefined,
  spellcheck = undefined,
  wrap = undefined,
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
   * Keep the textarea focusable while preventing user editing.
   *
   * @default false
   */
  readonly readOnly?: boolean;

  /**
   * Mark the textarea as required for native constraint validation.
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
   * Suggested visible text row count.
   *
   * @default undefined
   */
  readonly rows?: number;

  /**
   * Suggested visible text column count.
   *
   * @default undefined
   */
  readonly cols?: number;

  /**
   * Minimum accepted string length for native constraint validation.
   *
   * @default undefined
   */
  readonly minlength?: number;

  /**
   * Maximum accepted string length for native constraint validation.
   *
   * @default undefined
   */
  readonly maxlength?: number;

  /**
   * Native spellcheck preference.
   *
   * @default undefined
   */
  readonly spellcheck?: boolean;

  /**
   * Native line-wrapping policy.
   *
   * @default undefined
   */
  readonly wrap?: TextareaWrap;

  /**
   * Accessible name when no label or `aria-labelledby` supplies one.
   *
   * @default undefined
   */
  readonly ariaLabel?: string;

  /**
   * Space-separated ids that label the textarea.
   *
   * @default undefined
   */
  readonly ariaLabelledby?: string;

  /**
   * Space-separated ids that describe the textarea.
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
  readonly ariaInvalid?: TextareaAriaInvalid;
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

const element = useTemplateRef<HTMLTextAreaElement>("element");
const controlId = useDeterministicId({ id: () => id, hint: "textarea" });
const composing = ref(false);
const composingValue = ref<string | undefined>(undefined);
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
const renderedValue = computed(() =>
  composing.value ? (composingValue.value ?? element.value?.value ?? value.value) : value.value,
);
const dataEmpty = computed(() => (renderedValue.value.length === 0 ? "true" : "false"));

function syncNativeValue(): void {
  if (composing.value) return;
  if (element.value === null) return;
  if (element.value.value !== value.value) element.value.value = value.value;
}

watchEffect(syncNativeValue);

watch(
  element,
  (textarea, _previous, onCleanup) => {
    const form = textarea?.form;
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

function readTextAreaValue(event: Event): string | undefined {
  if (!(event.currentTarget instanceof HTMLTextAreaElement)) return undefined;
  return event.currentTarget.value;
}

function onInput(event: Event): void {
  const next = readTextAreaValue(event);
  if (next === undefined) return;
  if (composing.value) composingValue.value = next;
  state.set(next);
  emit("input", next, event);
  void nextTick(syncNativeValue);
}

function onChange(event: Event): void {
  const next = readTextAreaValue(event);
  if (next === undefined) return;
  state.set(next);
  emit("change", next, event);
  void nextTick(syncNativeValue);
}

function onCompositionStart(event: CompositionEvent): void {
  composing.value = true;
  composingValue.value = element.value?.value ?? value.value;
  emit("compositionStart", value.value, event);
}

function onCompositionEnd(event: CompositionEvent): void {
  const next = readTextAreaValue(event);
  const composed = next ?? composingValue.value;
  composing.value = false;
  if (composed !== undefined && composed !== composingValue.value) state.set(composed);
  composingValue.value = undefined;
  emit("compositionEnd", composed ?? value.value, event);
  void nextTick(syncNativeValue);
}

function focus(options?: FocusOptions): void {
  element.value?.focus(options);
}

function select(): void {
  element.value?.select();
}

function setSelectionRange(
  selectionStart: number,
  selectionEnd: number,
  direction?: "backward" | "forward" | "none",
): void {
  element.value?.setSelectionRange(selectionStart, selectionEnd, direction);
}

function setValue(next: string): boolean {
  const changed = state.set(next);
  void nextTick(syncNativeValue);
  return changed;
}

type TextareaSetupExpose = Omit<TextareaExpose, "composing" | "value"> & {
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
  setSelectionRange,
  setValue,
  value,
} satisfies TextareaSetupExpose;

defineExpose(exposed);
</script>

<template>
  <textarea
    :id="controlId"
    ref="element"
    :name
    :value="renderedValue"
    :disabled
    :readonly="readOnly"
    :required
    :placeholder
    :autocomplete
    :rows
    :cols
    :minlength
    :maxlength
    :spellcheck
    :wrap
    :aria-label="ariaLabel"
    :aria-labelledby="ariaLabelledby"
    :aria-describedby="ariaDescribedby"
    :aria-errormessage="ariaInvalidValue === undefined ? undefined : ariaErrormessage"
    :aria-invalid="ariaInvalidValue"
    data-vize-ui="textarea"
    :data-state="dataState"
    :data-empty="dataEmpty"
    :data-composing="composing ? 'true' : 'false'"
    @input="onInput"
    @change="onChange"
    @compositionstart="onCompositionStart"
    @compositionend="onCompositionEnd"
  ></textarea>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
