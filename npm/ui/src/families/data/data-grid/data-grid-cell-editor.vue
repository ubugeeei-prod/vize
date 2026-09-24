<script setup lang="ts">
import { computed, onMounted, useTemplateRef } from "vue";

import { dataGridContext } from "./data-grid-context.ts";

const { label = undefined } = defineProps<{
  /**
   * Accessible name of the editor input.
   *
   * @default undefined
   */
  readonly label?: string;
}>();

const grid = dataGridContext.use();
const element = useTemplateRef<HTMLInputElement>("element");
const draft = computed(() => grid.editing.value?.draft ?? "");
const error = computed(() => grid.editing.value?.error ?? null);

onMounted(() => {
  element.value?.focus();
  element.value?.select();
});

function onInput(event: Event): void {
  const target = event.target;
  if (target instanceof HTMLInputElement) grid.setDraft(target.value);
}

defineExpose({ element });
</script>

<template>
  <input
    ref="element"
    type="text"
    :value="draft"
    :aria-label="label"
    :aria-invalid="error ? 'true' : undefined"
    data-vize-ui="data-grid-cell-editor"
    part="cell-editor"
    :data-error="error ?? undefined"
    @input="onInput"
  />
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
