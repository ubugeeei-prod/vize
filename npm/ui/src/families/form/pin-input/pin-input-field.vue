<script setup lang="ts">
import { computed, onScopeDispose, useTemplateRef, watch } from "vue";

import { pinInputContext } from "./pin-input-context.ts";

const { index, ariaLabel = undefined } = defineProps<{
  /**
   * Zero-based position of this field in the code.
   *
   * @default required
   */
  readonly index: number;

  /**
   * Accessible name overriding the group's `getFieldLabel`.
   *
   * @default undefined
   */
  readonly ariaLabel?: string;
}>();

const context = pinInputContext.use();
const element = useTemplateRef<HTMLInputElement>("element");
const character = computed(() => context.characters.value[index] ?? "");
const filled = computed(() => character.value.length > 0);
const autocomplete = computed(() => (context.otp.value && index === 0 ? "one-time-code" : "off"));

watch(element, (input) => context.registerField(index, input), { flush: "sync", immediate: true });
onScopeDispose(() => context.registerField(index, null));

function syncNativeValue(): void {
  if (element.value !== null && element.value.value !== character.value) {
    element.value.value = character.value;
  }
}

function onInput(event: Event): void {
  if (!(event.currentTarget instanceof HTMLInputElement)) return;
  const text = event.currentTarget.value;
  // Typing over a filled field appends; keep only what is new relative to the old character.
  const typed =
    text.startsWith(character.value) && text.length > character.value.length
      ? text.slice(character.value.length)
      : text;
  const next = typed.length === 0 ? index : context.write(index, typed);
  queueMicrotask(() => {
    syncNativeValue();
    if (typed.length > 0) context.focusField(next);
  });
}

function onPaste(event: ClipboardEvent): void {
  const text = event.clipboardData?.getData("text") ?? "";
  if (text.length === 0) return;
  event.preventDefault();
  const next = context.write(index, text);
  queueMicrotask(() => context.focusField(next));
}

function onKeydown(event: KeyboardEvent): void {
  if (event.altKey || event.metaKey || event.ctrlKey || event.isComposing) return;
  const key = event.key;
  let target: number | undefined;
  if (key === "Backspace") target = context.remove(index, "backward");
  else if (key === "Delete") target = context.remove(index, "forward");
  else if (key === "ArrowLeft") target = index - 1;
  else if (key === "ArrowRight") target = index + 1;
  else if (key === "Home") target = 0;
  else if (key === "End")
    target = Math.min(context.characters.value.length, context.length.value - 1);
  if (target === undefined) return;
  event.preventDefault();
  const focusTarget = target;
  queueMicrotask(() => {
    syncNativeValue();
    context.focusField(focusTarget);
  });
}

function onFocus(event: FocusEvent): void {
  if (event.currentTarget instanceof HTMLInputElement) event.currentTarget.select();
}
</script>

<template>
  <input
    :id="`${context.baseId.value}-${index}`"
    ref="element"
    :type="context.mask.value ? 'password' : 'text'"
    :value="character"
    :inputmode="context.type.value === 'numeric' ? 'numeric' : 'text'"
    :autocomplete
    autocapitalize="off"
    autocorrect="off"
    spellcheck="false"
    :placeholder="context.placeholder.value"
    :disabled="context.disabled.value"
    :required="context.required.value && !context.complete.value"
    :aria-label="ariaLabel ?? context.fieldLabel(index)"
    :aria-describedby="context.ariaDescribedby.value"
    :aria-invalid="context.ariaInvalid.value"
    :aria-errormessage="
      context.ariaInvalid.value === undefined ? undefined : context.ariaErrormessage.value
    "
    part="field"
    data-vize-ui="pin-input-field"
    :data-index="index"
    :data-filled="filled ? 'true' : 'false'"
    @input="onInput"
    @paste="onPaste"
    @keydown="onKeydown"
    @focus="onFocus"
  />
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
