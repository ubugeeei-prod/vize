<script setup lang="ts">
import { computed, onScopeDispose, ref, useTemplateRef, watch } from "vue";

import { numberFieldContext } from "./number-field-context.ts";
import type { NumberFieldInputEmits, NumberFieldInputProps } from "./number-field-types.ts";

const { placeholder = undefined, autocomplete = "off" } = defineProps<NumberFieldInputProps>();

const emit = defineEmits<NumberFieldInputEmits>();

const context = numberFieldContext.use();
const element = useTemplateRef<HTMLInputElement>("element");
const composing = ref(false);
const focused = ref(false);
const allowsNegative = computed(() => context.bounds.value.min < 0);
const allowsFraction = computed(
  () =>
    context.parser.value.resolvedOptions.maximumFractionDigits !== 0 ||
    context.parser.value.resolvedOptions.style === "percent",
);
const inputMode = computed(() => {
  // Mobile numeric keypads omit the minus sign, so signed fields use a text keyboard.
  if (allowsNegative.value) return "text";
  return allowsFraction.value ? "decimal" : "numeric";
});
const ariaValueNow = computed(() => context.value.value ?? undefined);
const ariaValueMin = computed(() =>
  Number.isFinite(context.bounds.value.min) ? context.bounds.value.min : undefined,
);
const ariaValueMax = computed(() =>
  Number.isFinite(context.bounds.value.max) ? context.bounds.value.max : undefined,
);
const ariaValueText = computed(() => {
  const formatted =
    context.value.value === null ? "" : context.parser.value.format(context.value.value);
  return formatted.length === 0 ? undefined : formatted;
});

watch(
  element,
  (input) => {
    context.inputElement.value = input;
  },
  { flush: "sync", immediate: true },
);
onScopeDispose(() => {
  context.inputElement.value = null;
});

function syncNativeText(): void {
  if (element.value !== null && element.value.value !== context.inputText.value) {
    element.value.value = context.inputText.value;
  }
}

function onInput(event: Event): void {
  if (!(event.currentTarget instanceof HTMLInputElement)) return;
  const text = event.currentTarget.value;
  if (composing.value) return;
  if (context.readOnly.value) {
    syncNativeText();
    return;
  }
  if (context.parser.value.isPartial(text, { allowNegative: allowsNegative.value })) {
    context.setDraft(text);
    return;
  }
  // Reject characters that can never form a number and keep the previous text.
  event.currentTarget.value = context.inputText.value;
  emit("reject", text, event);
}

function onCompositionStart(): void {
  composing.value = true;
}

function onCompositionEnd(event: CompositionEvent): void {
  composing.value = false;
  onInput(event);
}

function keyboardAction(key: string): (() => void) | undefined {
  const bounds = context.bounds.value;
  if (key === "ArrowUp") return () => context.step("increment", "step", "keyboard");
  if (key === "ArrowDown") return () => context.step("decrement", "step", "keyboard");
  if (key === "PageUp") return () => context.step("increment", "largeStep", "keyboard");
  if (key === "PageDown") return () => context.step("decrement", "largeStep", "keyboard");
  // Home/End keep native caret movement when the field is unbounded.
  if (key === "Home" && Number.isFinite(bounds.min)) {
    return () => context.stepToBound("min", "keyboard");
  }
  if (key === "End" && Number.isFinite(bounds.max)) {
    return () => context.stepToBound("max", "keyboard");
  }
  return undefined;
}

function onKeydown(event: KeyboardEvent): void {
  if (event.isComposing || event.altKey || event.metaKey || event.ctrlKey) return;
  if (event.key === "Enter") {
    // Commit before implicit form submission reads the hidden value.
    if (!context.disabled.value && !context.readOnly.value) context.commit("enter");
    return;
  }
  const action = keyboardAction(event.key);
  if (action === undefined) return;
  event.preventDefault();
  action();
}

function onFocus(): void {
  focused.value = true;
}

function onBlur(): void {
  focused.value = false;
  if (!context.disabled.value && !context.readOnly.value) context.commit("blur");
}

function onWheel(event: WheelEvent): void {
  if (!focused.value || !context.allowWheel.value || event.deltaY === 0) return;
  if (context.disabled.value || context.readOnly.value) return;
  event.preventDefault();
  context.step(event.deltaY < 0 ? "increment" : "decrement", "step", "wheel");
}

// Wheel listeners must be non-passive to cancel page scrolling, which template
// listeners cannot declare, so the listener is attached only while opted in.
watch(
  [element, () => context.allowWheel.value],
  ([input, allowWheel], _previous, onCleanup) => {
    if (input === null || !allowWheel) return;
    input.addEventListener("wheel", onWheel, { passive: false });
    onCleanup(() => input.removeEventListener("wheel", onWheel));
  },
  { flush: "post", immediate: true },
);
</script>

<template>
  <input
    :id="context.inputId.value"
    ref="element"
    type="text"
    role="spinbutton"
    :value="context.inputText.value"
    :form="context.form.value"
    :inputmode="inputMode"
    :placeholder
    :autocomplete
    autocorrect="off"
    spellcheck="false"
    :disabled="context.disabled.value"
    :readonly="context.readOnly.value"
    :required="context.required.value"
    :aria-valuenow="ariaValueNow"
    :aria-valuemin="ariaValueMin"
    :aria-valuemax="ariaValueMax"
    :aria-valuetext="ariaValueText"
    :aria-label="context.ariaLabel.value"
    :aria-labelledby="context.ariaLabelledby.value"
    :aria-describedby="context.ariaDescribedby.value"
    :aria-errormessage="
      context.ariaInvalid.value === undefined ? undefined : context.ariaErrormessage.value
    "
    :aria-invalid="context.ariaInvalid.value"
    :aria-readonly="context.readOnly.value ? 'true' : undefined"
    :aria-required="context.required.value ? 'true' : undefined"
    part="input"
    data-vize-ui="number-field-input"
    :data-state="context.state.value"
    @input="onInput"
    @compositionstart="onCompositionStart"
    @compositionend="onCompositionEnd"
    @keydown="onKeydown"
    @focus="onFocus"
    @blur="onBlur"
  />
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
