<script setup lang="ts">
import type { ChatProps } from "./types.ts";

const { label = "Conversation" } = defineProps<ChatProps>();

defineSlots<{
  /** Renders messages and one composer. */
  default(): unknown;
}>();
</script>

<template>
  <div
    role="log"
    data-vize-ui="chat"
    aria-live="polite"
    aria-relevant="additions"
    :aria-label="label"
  >
    <slot />
  </div>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
