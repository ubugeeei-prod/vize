<script setup lang="ts">
import { computed, onScopeDispose, useTemplateRef, watch } from "vue";

import { editableContext } from "./editable-context.ts";

defineSlots<{
  /** Preview contents; defaults to the value or the placeholder. */
  default?(props: { readonly value: string; readonly empty: boolean }): unknown;
}>();

const context = editableContext.use();
const element = useTemplateRef<HTMLSpanElement>("element");
const empty = computed(() => context.value.value.length === 0);
// Keyboard activation (Enter/F2) works in every mode; `activationMode` only adds pointer or focus activation.
const activatable = computed(() => context.interactive.value);

watch(element, (preview) => context.registerPreview(preview), { flush: "sync", immediate: true });
onScopeDispose(() => context.registerPreview(null));

function onFocus(): void {
  if (context.consumeFocusReturn()) return;
  if (context.activationMode.value === "focus") context.edit();
}

function onClick(): void {
  if (context.activationMode.value === "click") context.edit();
}

function onDblclick(): void {
  if (context.activationMode.value === "dblclick") context.edit();
}

function onKeydown(event: KeyboardEvent): void {
  if (!activatable.value || (event.key !== "Enter" && event.key !== "F2")) return;
  event.preventDefault();
  context.edit();
}
</script>

<template>
  <span
    :id="`${context.inputId.value}-preview`"
    ref="element"
    role="button"
    tabindex="0"
    :hidden="context.editing.value"
    :aria-labelledby="context.ariaLabelledby.value"
    :aria-disabled="activatable ? undefined : 'true'"
    part="preview"
    data-vize-ui="editable-preview"
    :data-empty="empty ? 'true' : undefined"
    :data-state="context.state.value"
    @focus="onFocus"
    @click="onClick"
    @dblclick="onDblclick"
    @keydown="onKeydown"
  >
    <slot :value="context.value.value" :empty="empty">{{
      empty ? (context.placeholder.value ?? "") : context.value.value
    }}</slot>
  </span>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
