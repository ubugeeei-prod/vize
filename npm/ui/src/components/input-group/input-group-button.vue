<script setup lang="ts">
import { computed, inject } from "vue";

import { inputGroupKey } from "./context.ts";
import type { InputGroupButtonProps } from "./types.ts";

const { disabled = false } = defineProps<InputGroupButtonProps>();

const emit = defineEmits<{
  /**
   * Fired when the button is activated and the group is available.
   *
   * Carries the click `MouseEvent`.
   */
  press: [event: MouseEvent];
}>();

defineSlots<{
  /** Renders the button label. */
  default(): unknown;
}>();

const group = inject(inputGroupKey, null);
const off = computed(() => disabled || (group?.disabled.value ?? false));

function onClick(event: MouseEvent): void {
  if (off.value) {
    event.preventDefault();
    return;
  }
  emit("press", event);
}
</script>

<template>
  <button
    type="button"
    data-vize-ui="input-group-button"
    :disabled="off"
    :data-state="off ? 'disabled' : 'idle'"
    @click="onClick"
  >
    <slot />
  </button>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
