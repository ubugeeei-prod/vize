<script setup lang="ts">
import { emojiPickerContext } from "./emoji-picker-context.ts";

defineSlots<{
  /** Message shown while no emoji is visible. Receives the search text. */
  default?(props: { readonly search: string }): unknown;
}>();

const context = emojiPickerContext.use();
</script>

<template>
  <div
    role="status"
    data-vize-ui="emoji-picker-empty"
    part="empty"
    :hidden="context.empty.value ? undefined : true"
  >
    <slot v-if="context.empty.value" :search="context.search.value" />
  </div>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
