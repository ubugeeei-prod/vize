<script setup lang="ts">
import { onScopeDispose, useTemplateRef, watch } from "vue";

import { editableContext } from "./editable-context.ts";

const context = editableContext.use();
const element = useTemplateRef<HTMLInputElement>("element");

watch(element, (input) => context.registerInput(input), { flush: "sync", immediate: true });
onScopeDispose(() => context.registerInput(null));

function onInput(event: Event): void {
  if (event.currentTarget instanceof HTMLInputElement) context.setDraft(event.currentTarget.value);
}

function onKeydown(event: KeyboardEvent): void {
  if (event.isComposing) return;
  if (event.key === "Escape") {
    event.preventDefault();
    context.cancel();
  } else if (event.key === "Enter" && context.submitOnEnter.value) {
    event.preventDefault();
    context.submit();
  }
}

function onBlur(): void {
  if (context.editing.value && context.submitOnBlur.value) context.submit();
}
</script>

<template>
  <input
    :id="context.inputId.value"
    ref="element"
    type="text"
    :value="context.draft.value"
    :hidden="!context.editing.value"
    :disabled="context.disabled.value"
    :readonly="context.readOnly.value"
    :placeholder="context.placeholder.value"
    :maxlength="context.maxLength.value"
    :aria-label="context.ariaLabel.value"
    :aria-labelledby="context.ariaLabelledby.value"
    :aria-describedby="context.ariaDescribedby.value"
    :aria-errormessage="
      context.ariaInvalid.value === undefined ? undefined : context.ariaErrormessage.value
    "
    :aria-invalid="context.ariaInvalid.value"
    part="input"
    data-vize-ui="editable-input"
    :data-state="context.state.value"
    @input="onInput"
    @keydown="onKeydown"
    @blur="onBlur"
  />
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
