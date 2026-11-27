<script setup lang="ts">
import { computed, inject } from "vue";

import { inputGroupKey } from "./context.ts";
import type { InputGroupTextareaProps } from "./types.ts";

const {
  modelValue = "",
  rows = 2,
  disabled = false,
  name = undefined,
} = defineProps<InputGroupTextareaProps>();

const emit = defineEmits<{
  /**
   * Fired when the user edits the textarea.
   *
   * Carries the next string.
   */
  "update:modelValue": [value: string];
}>();

const group = inject(inputGroupKey, null);
const off = computed(() => disabled || (group?.disabled.value ?? false));

function onInput(event: Event): void {
  if (event.target instanceof HTMLTextAreaElement) emit("update:modelValue", event.target.value);
}
</script>

<template>
  <textarea
    data-vize-ui="input-group-textarea"
    :name="name"
    :rows="rows"
    :value="modelValue"
    :disabled="off"
    :data-state="off ? 'disabled' : 'idle'"
    @input="onInput"
  />
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
