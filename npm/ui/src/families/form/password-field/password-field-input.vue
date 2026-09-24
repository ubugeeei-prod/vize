<script setup lang="ts">
import { onScopeDispose, useTemplateRef, watch } from "vue";

import { passwordFieldContext } from "./password-field-context.ts";

const {
  placeholder = undefined,
  minlength = undefined,
  maxlength = undefined,
} = defineProps<{
  /**
   * Native placeholder shown while empty.
   *
   * @default undefined
   */
  readonly placeholder?: string;

  /**
   * Native minimum length for constraint validation.
   *
   * @default undefined
   */
  readonly minlength?: number;

  /**
   * Native maximum length.
   *
   * @default undefined
   */
  readonly maxlength?: number;
}>();

const context = passwordFieldContext.use();
const element = useTemplateRef<HTMLInputElement>("element");

watch(element, (input) => context.registerInput(input), { flush: "sync", immediate: true });
onScopeDispose(() => context.registerInput(null));

function onInput(event: Event): void {
  if (!(event.currentTarget instanceof HTMLInputElement)) return;
  context.setValue(event.currentTarget.value);
  // Controlled parents may reject the change; keep the DOM on the model value.
  const input = event.currentTarget;
  queueMicrotask(() => {
    if (input.value !== context.value.value) input.value = context.value.value;
  });
}

function onKey(event: KeyboardEvent): void {
  // `getModifierState` reflects the lock state after this key event is applied.
  if (typeof event.getModifierState === "function") {
    context.setCapsLock(event.getModifierState("CapsLock"));
  }
}

function onBlur(): void {
  context.setCapsLock(false);
}
</script>

<template>
  <input
    :id="context.inputId.value"
    ref="element"
    :type="context.visible.value ? 'text' : 'password'"
    :name="context.name.value"
    :value="context.value.value"
    :autocomplete="context.autocomplete.value"
    autocapitalize="off"
    autocorrect="off"
    spellcheck="false"
    :placeholder
    :minlength
    :maxlength
    :disabled="context.disabled.value"
    :readonly="context.readOnly.value"
    :required="context.required.value"
    :aria-label="context.ariaLabel.value"
    :aria-labelledby="context.ariaLabelledby.value"
    :aria-describedby="context.ariaDescribedby.value"
    :aria-errormessage="
      context.ariaInvalid.value === undefined ? undefined : context.ariaErrormessage.value
    "
    :aria-invalid="context.ariaInvalid.value"
    part="input"
    data-vize-ui="password-field-input"
    :data-visible="context.visible.value ? 'true' : 'false'"
    @input="onInput"
    @keydown="onKey"
    @keyup="onKey"
    @blur="onBlur"
  />
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
