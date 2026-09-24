<script setup lang="ts">
import { tagsInputItemContext } from "./tags-input-context.ts";
import type { TagsInputItemSlotState } from "./tags-input-types.ts";

defineSlots<{
  /** Custom tag text. Receives the owning item state; defaults to the resolved tag text. */
  default?(props: TagsInputItemSlotState<unknown>): unknown;
}>();

const item = tagsInputItemContext.use();
</script>

<template>
  <span data-vize-ui="tags-input-item-text" part="item-text">
    <slot v-bind="item.slotState.value">{{ item.slotState.value.text }}</slot>
  </span>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
