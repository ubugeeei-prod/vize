<script setup lang="ts">
import { computed, inject } from "vue";

import { segmentedKey } from "./context.ts";
import type { SegmentedItemProps } from "./types.ts";

const { value, disabled = false } = defineProps<SegmentedItemProps>();

defineSlots<{
  /** Renders the option label. */
  default(): unknown;
}>();

const group = inject(segmentedKey, null);
const off = computed(() => disabled || (group?.disabled.value ?? false));
const selected = computed(() => group?.model.value === value);

if (!disabled) group?.claim(value);

const tabIndex = computed(() => {
  if (off.value) return -1;
  if (selected.value) return 0;
  if (group?.model.value === "" && group.fallback.value === value) return 0;
  return -1;
});

function onClick(): void {
  if (off.value) return;
  group?.choose(value);
}
</script>

<template>
  <button
    type="button"
    role="radio"
    data-vize-ui="segmented-item"
    :data-value="value"
    :aria-checked="selected ? 'true' : 'false'"
    :aria-disabled="off ? 'true' : undefined"
    :disabled="off"
    :tabindex="tabIndex"
    @click="onClick"
  >
    <slot />
  </button>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
