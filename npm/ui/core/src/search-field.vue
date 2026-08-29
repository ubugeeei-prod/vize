<script setup lang="ts">
import { computed, nextTick, ref, useTemplateRef, watch, watchEffect } from "vue";

import { useControllableState } from "./controllable-state.ts";
import { deriveDeterministicId, useDeterministicId } from "./deterministic-id.ts";
import type {
  SearchFieldAriaInvalid,
  SearchFieldClearSlotState,
  SearchFieldClearVisibility,
  SearchFieldEnterKeyHint,
  SearchFieldExpose,
  SearchFieldInputMode,
} from "./search-field-types.ts";

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
  inputMode = "search",
  enterKeyHint = "search",
  showClear = "auto",
  clearLabel = "Clear search",
  ariaLabel = undefined,
  ariaLabelledby = undefined,
  ariaDescribedby = undefined,
  ariaErrormessage = undefined,
  ariaInvalid = false,
} = defineProps<{
  /** Consumer-owned input id; nullish values use a deterministic fallback. @default undefined */
  readonly id?: string | null;
  /** Native form field name. @default undefined */
  readonly name?: string;
  /** Controlled string value; undefined selects uncontrolled behavior. @default undefined */
  readonly modelValue?: string;
  /** Initial uncontrolled value and form-reset target. @default "" */
  readonly defaultValue?: string;
  /** Disable editing, clearing, focus, and native form submission. @default false */
  readonly disabled?: boolean;
  /** Keep focusability while preventing user editing and clearing. @default false */
  readonly readOnly?: boolean;
  /** Mark the native search input as required. @default false */
  readonly required?: boolean;
  /** Native placeholder text. @default undefined */
  readonly placeholder?: string;
  /** Native autocomplete hint. @default undefined */
  readonly autocomplete?: string;
  /** Native virtual-keyboard input mode. @default "search" */
  readonly inputMode?: SearchFieldInputMode;
  /** Native virtual-keyboard enter key hint. @default "search" */
  readonly enterKeyHint?: SearchFieldEnterKeyHint;
  /** Clear button visibility policy. @default "auto" */
  readonly showClear?: SearchFieldClearVisibility;
  /** Accessible name for the default clear button. @default "Clear search" */
  readonly clearLabel?: string;
  /** Accessible name when no label or aria-labelledby supplies one. @default undefined */
  readonly ariaLabel?: string;
  /** Space-separated ids that label the search input. @default undefined */
  readonly ariaLabelledby?: string;
  /** Space-separated ids that describe the search input. @default undefined */
  readonly ariaDescribedby?: string;
  /** Id of the validation error message used while invalid. @default undefined */
  readonly ariaErrormessage?: string;
  /** Invalid state announced to assistive technology. @default false */
  readonly ariaInvalid?: SearchFieldAriaInvalid;
}>();

defineSlots<{
  /** Replaces the default clear button contents with availability state. */
  clear?(props: SearchFieldClearSlotState): unknown;
}>();

const emit = defineEmits<{
  /** Fired when the value requests a new controlled string. */
  "update:modelValue": [value: string];
  /** Fired after the default clear button clears the field. */
  clear: [value: "", nativeEvent: MouseEvent];
  /** Fired after a native change/commit with the current string and native Event. */
  change: [value: string, nativeEvent: Event];
  /** Fired when IME composition ends. */
  compositionEnd: [value: string, nativeEvent: CompositionEvent];
  /** Fired when IME composition starts. */
  compositionStart: [value: string, nativeEvent: CompositionEvent];
  /** Fired after a native input event with the next string and native Event. */
  input: [value: string, nativeEvent: Event];
  /** Fired after a native search event with the committed search string. */
  search: [value: string, nativeEvent: Event];
}>();

const element = useTemplateRef<HTMLInputElement>("element");
const controlId = useDeterministicId({ id: () => id, hint: "search" });
const clearId = computed(() => deriveDeterministicId(controlId.value, "clear"));
const composing = ref(false);
const composingValue = ref<string | undefined>(undefined);
const state = useControllableState({
  value: () => modelValue,
  defaultValue: () => defaultValue,
  onChange: (value) => emit("update:modelValue", value),
});
const value = state.value;
const renderedValue = computed(() =>
  composing.value ? (composingValue.value ?? element.value?.value ?? value.value) : value.value,
);
const empty = computed(() => renderedValue.value.length === 0);
const ariaInvalidValue = computed(() => {
  if (ariaInvalid === false) return undefined;
  return ariaInvalid === true ? "true" : ariaInvalid;
});
const dataState = computed(() => {
  if (disabled) return "disabled";
  return readOnly ? "readonly" : "editable";
});
const dataEmpty = computed(() => (empty.value ? "true" : "false"));
const clearVisible = computed(() => {
  if (showClear === "never") return false;
  return showClear === "always" || !empty.value;
});
const clearDisabled = computed(() => disabled || readOnly || empty.value);
const clearSlotState = computed((): SearchFieldClearSlotState => ({
  disabled: clearDisabled.value,
  empty: empty.value,
}));
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

function readSearchValue(event: Event): string | undefined {
  if (!(event.currentTarget instanceof HTMLInputElement)) return undefined;
  return event.currentTarget.value;
}

function onInput(event: Event): void {
  const next = readSearchValue(event);
  if (next === undefined) return;
  if (composing.value) composingValue.value = next;
  state.set(next);
  emit("input", next, event);
  void nextTick(syncNativeValue);
}

function onChange(event: Event): void {
  const next = readSearchValue(event);
  if (next === undefined) return;
  state.set(next);
  emit("change", next, event);
  void nextTick(syncNativeValue);
}

function onSearch(event: Event): void {
  const next = readSearchValue(event);
  if (next === undefined) return;
  state.set(next);
  emit("search", next, event);
  void nextTick(syncNativeValue);
}

function onCompositionStart(event: CompositionEvent): void {
  composing.value = true;
  composingValue.value = element.value?.value ?? value.value;
  emit("compositionStart", value.value, event);
}

function onCompositionEnd(event: CompositionEvent): void {
  const next = readSearchValue(event);
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

function setValue(next: string): boolean {
  const changed = state.set(next);
  void nextTick(syncNativeValue);
  return changed;
}

function clear(): boolean {
  if (clearDisabled.value) return false;
  const changed = state.set("");
  void nextTick(() => {
    syncNativeValue();
    focus();
  });
  return changed;
}

function onClearClick(event: MouseEvent): void {
  if (!clear()) return;
  emit("clear", "", event);
}

type SearchFieldSetupExpose = Omit<SearchFieldExpose, "composing" | "value"> & {
  readonly composing: typeof composing;
  readonly element: typeof element;
  readonly value: typeof value;
};

const exposed = {
  clear,
  composing,
  element,
  focus,
  reset: state.reset,
  select,
  setValue,
  value,
} satisfies SearchFieldSetupExpose;

defineExpose(exposed);
</script>

<template>
  <div
    role="search"
    data-vize-ui="search-field"
    :data-state="dataState"
    :data-empty="dataEmpty"
    :data-composing="composing ? 'true' : 'false'"
  >
    <input
      :id="controlId"
      ref="element"
      :name
      type="search"
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
      data-vize-ui="search-field-input"
      :data-state="dataState"
      :data-empty="dataEmpty"
      :data-composing="composing ? 'true' : 'false'"
      @input="onInput"
      @change="onChange"
      @search="onSearch"
      @compositionstart="onCompositionStart"
      @compositionend="onCompositionEnd"
    />
    <button
      v-if="clearVisible"
      :id="clearId"
      type="button"
      :disabled="clearDisabled"
      :aria-label="clearLabel"
      data-vize-ui="search-field-clear"
      :data-empty="dataEmpty"
      @click="onClearClick"
    >
      <slot name="clear" v-bind="clearSlotState" />
    </button>
  </div>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
