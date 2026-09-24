<script setup lang="ts">
import { computed, onMounted, onUnmounted, useTemplateRef } from "vue";

import { comboboxContext } from "./combobox-context.ts";

defineSlots<{
  /** Chips, the input, and the toggle button. The popup is positioned against this box. */
  default(): unknown;
}>();

const context = comboboxContext.use();
const element = useTemplateRef<HTMLDivElement>("element");
const handlers = computed(() => ({ onPointerdown }));

onMounted(() => {
  context.anchorElement.value = element.value;
});

onUnmounted(() => {
  if (context.anchorElement.value === element.value) context.anchorElement.value = null;
});

// Presses on the anchor's padding or chip area keep focus in the input.
function onPointerdown(event: PointerEvent): void {
  const input = context.inputElement.value;
  if (input === null || event.target === input || context.disabled.value) return;
  if (event.target instanceof Element && event.target.closest("button, a, input") !== null) return;
  event.preventDefault();
  input.focus();
}

defineExpose({ element });
</script>

<template>
  <div
    ref="element"
    v-bind="handlers"
    data-vize-ui="combobox-anchor"
    part="anchor"
    :data-state="context.open.value ? 'open' : 'closed'"
    :data-disabled="context.disabled.value ? 'true' : undefined"
  >
    <slot />
  </div>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
