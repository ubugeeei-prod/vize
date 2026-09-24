<script setup lang="ts">
import { computed, onMounted, onUnmounted, useTemplateRef } from "vue";

import { comboboxContext } from "./combobox-context.ts";

const { ariaLabel = "Show suggestions" } = defineProps<{
  /**
   * Accessible name of the toggle button.
   *
   * @default "Show suggestions"
   */
  readonly ariaLabel?: string;
}>();

defineSlots<{
  /** Decorative toggle content such as a chevron. Receives the open state. */
  default(props: { readonly open: boolean }): unknown;
}>();

const context = comboboxContext.use();
const element = useTemplateRef<HTMLButtonElement>("element");
const handlers = computed(() => ({
  onClick,
  onPointerdown: (event: PointerEvent) => event.preventDefault(),
}));

onMounted(() => {
  context.toggleElement.value = element.value;
});

onUnmounted(() => {
  if (context.toggleElement.value === element.value) context.toggleElement.value = null;
});

function onClick(event: MouseEvent): void {
  if (context.disabled.value || context.readonly.value) return;
  context.setOpen(!context.open.value, event);
  context.inputElement.value?.focus();
}

defineExpose({ element });
</script>

<template>
  <button
    ref="element"
    v-bind="handlers"
    type="button"
    tabindex="-1"
    :disabled="context.disabled.value"
    :aria-label="ariaLabel"
    :aria-expanded="context.open.value ? 'true' : 'false'"
    :aria-controls="context.open.value ? context.listboxId.value : undefined"
    data-vize-ui="combobox-trigger"
    part="trigger"
    :data-state="context.open.value ? 'open' : 'closed'"
  >
    <slot :open="context.open.value" />
  </button>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
