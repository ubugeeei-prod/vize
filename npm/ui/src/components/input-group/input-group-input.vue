<script setup lang="ts">
import { computed, inject } from "vue";

import { inputGroupKey } from "./context.ts";
import type { InputGroupInputProps } from "./types.ts";

const {
  modelValue = "",
  type = "text",
  disabled = false,
  name = undefined,
} = defineProps<InputGroupInputProps>();

const emit = defineEmits<{
  /**
   * Fired when the user edits the field.
   *
   * Carries the next string.
   */
  "update:modelValue": [value: string];
}>();

const group = inject(inputGroupKey, null);
const off = computed(() => disabled || (group?.disabled.value ?? false));

function onInput(event: Event): void {
  if (event.target instanceof HTMLInputElement) emit("update:modelValue", event.target.value);
}
</script>

<template>
  <!-- The consumer labels this native root through fallthrough attrs or an external label. -->
  <!-- eslint-disable-next-line a11y/form-control-has-label -->
  <input
    data-vize-ui="input-group-input"
    :type
    :name
    :value="modelValue"
    :disabled="off"
    :data-state="off ? 'disabled' : 'idle'"
    @input="onInput"
  />
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
