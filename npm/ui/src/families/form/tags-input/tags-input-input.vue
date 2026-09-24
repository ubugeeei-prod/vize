<script setup lang="ts">
import { computed, useTemplateRef } from "vue";

import { tagsInputContext } from "./tags-input-context.ts";
import type { TagsInputInputExpose } from "./tags-input-types.ts";

const { placeholder = undefined, autocomplete = "off" } = defineProps<{
  /**
   * Native placeholder shown while the input is empty.
   *
   * @default undefined
   */
  readonly placeholder?: string;

  /**
   * Native autocomplete hint.
   *
   * @default "off"
   */
  readonly autocomplete?: string;
}>();

const context = tagsInputContext.use();
const element = useTemplateRef<HTMLInputElement>("element");
const nativeRequired = computed(
  () => context.required.value && context.tags.value.length === 0 && !context.disabled.value,
);

function caretAtStart(input: HTMLInputElement): boolean {
  return input.selectionStart === 0 && input.selectionEnd === 0;
}

function onKeydown(event: KeyboardEvent): void {
  // IME composition owns Enter and delimiters until the candidate is committed.
  if (event.isComposing || event.keyCode === 229 || event.defaultPrevented) return;
  const input = event.currentTarget as HTMLInputElement;
  const count = context.tags.value.length;
  if (event.key === "Enter") {
    if (context.inputValue.value.trim().length === 0) return;
    event.preventDefault();
    context.commitInput("enter");
    return;
  }
  if (context.isDelimiterKey(event.key)) {
    event.preventDefault();
    context.commitInput("delimiter");
    return;
  }
  const backward = context.direction.value === "rtl" ? "ArrowRight" : "ArrowLeft";
  if ((event.key === "Backspace" || event.key === backward) && count > 0 && caretAtStart(input)) {
    if (context.focusItem(count - 1)) event.preventDefault();
  }
}

function onInput(event: Event): void {
  const input = event.currentTarget as HTMLInputElement;
  context.handleInputText(input.value);
  if (input.value !== context.inputValue.value) input.value = context.inputValue.value;
}

function onPaste(event: ClipboardEvent): void {
  const input = event.currentTarget as HTMLInputElement;
  const pasted = event.clipboardData?.getData("text") ?? "";
  const start = input.selectionStart ?? input.value.length;
  const end = input.selectionEnd ?? start;
  if (context.handlePaste(pasted, start, end)) event.preventDefault();
}

function onBlur(): void {
  if (context.addOnBlur.value) context.commitInput("blur");
}

function focus(options?: FocusOptions): void {
  element.value?.focus(options);
}

type TagsInputInputSetupExpose = Omit<TagsInputInputExpose, "element"> & {
  readonly element: typeof element;
};

const exposed = { element, focus } satisfies TagsInputInputSetupExpose;

defineExpose(exposed);
</script>

<template>
  <input
    :id="context.inputId.value"
    ref="element"
    type="text"
    :value="context.inputValue.value"
    :placeholder
    :autocomplete
    :disabled="context.disabled.value"
    :readonly="context.readonly.value"
    :required="nativeRequired"
    :aria-required="context.required.value ? 'true' : undefined"
    :aria-label="context.ariaLabel.value"
    :aria-labelledby="context.ariaLabelledby.value"
    :aria-describedby="context.ariaDescribedby.value"
    :aria-invalid="context.ariaInvalid.value"
    :aria-errormessage="
      context.ariaInvalid.value === undefined ? undefined : context.ariaErrormessage.value
    "
    data-vize-ui="tags-input-input"
    part="input"
    @keydown="onKeydown"
    @input="onInput"
    @paste="onPaste"
    @blur="onBlur"
  />
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
