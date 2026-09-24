<script setup lang="ts">
import { computed } from "vue";

import { comboboxChipContext } from "./combobox-context.ts";

const { ariaLabel = undefined } = defineProps<{
  /**
   * Accessible name. Defaults to `Remove <chip text>`.
   *
   * @default undefined
   */
  readonly ariaLabel?: string;
}>();

defineSlots<{
  /** Decorative remove icon. */
  default?(): unknown;
}>();

const chip = comboboxChipContext.use();
const label = computed(() => ariaLabel ?? `Remove ${chip.text.value}`);
const handlers = computed(() => ({
  onClick: () => chip.remove(),
  onPointerdown: (event: PointerEvent) => event.preventDefault(),
}));
</script>

<template>
  <button
    v-bind="handlers"
    type="button"
    tabindex="-1"
    :disabled="chip.disabled.value"
    :aria-label="label"
    data-vize-ui="combobox-chip-remove"
    part="chip-remove"
  >
    <slot />
  </button>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
