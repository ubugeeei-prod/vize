<script setup lang="ts">
import { computed } from "vue";

import { tagsInputContext, tagsInputItemContext } from "./tags-input-context.ts";
import type { TagsInputItemSlotState } from "./tags-input-types.ts";

const { label = "Remove", ariaLabel = undefined } = defineProps<{
  /**
   * Accessible name prefix followed by the tag text, e.g. "Remove vue".
   *
   * @default "Remove"
   */
  readonly label?: string;

  /**
   * Full accessible name override.
   *
   * @default undefined
   */
  readonly ariaLabel?: string;
}>();

defineSlots<{
  /** Button content such as an icon. Receives the owning item state. */
  default?(props: TagsInputItemSlotState<unknown>): unknown;
}>();

const context = tagsInputContext.use();
const item = tagsInputItemContext.use();
const unavailable = computed(
  () => item.slotState.value.disabled || context.readonly.value || context.disabled.value,
);
const name = computed(() => ariaLabel ?? `${label} ${item.slotState.value.text}`);

function onClick(event: MouseEvent): void {
  event.preventDefault();
  if (!unavailable.value) item.remove("delete-button");
}
</script>

<template>
  <button
    type="button"
    tabindex="-1"
    :aria-label="name"
    :disabled="unavailable"
    data-vize-ui="tags-input-item-delete"
    part="item-delete"
    @click="onClick"
  >
    <slot v-bind="item.slotState.value" />
  </button>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
