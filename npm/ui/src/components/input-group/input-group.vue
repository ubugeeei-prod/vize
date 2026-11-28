<script setup lang="ts">
import { computed, provide } from "vue";

import { inputGroupKey } from "./context.ts";
import type { InputGroupProps } from "./types.ts";

const { label, disabled = false } = defineProps<InputGroupProps>();

defineSlots<{
  /** Renders addons, fields, and buttons. */
  default(): unknown;
}>();

provide(inputGroupKey, { disabled: computed(() => disabled) });
</script>

<template>
  <div
    role="group"
    data-vize-ui="input-group"
    :aria-label="label"
    :aria-disabled="disabled ? 'true' : undefined"
    :data-state="disabled ? 'disabled' : 'idle'"
  >
    <slot />
  </div>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
