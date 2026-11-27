<script setup lang="ts">
import type { ChatMessageProps } from "./types.ts";

const { author } = defineProps<ChatMessageProps>();

defineSlots<{
  /** Renders the message body. */
  default(): unknown;
}>();
</script>

<template>
  <article role="article" data-vize-ui="chat-message" :aria-label="author" :data-author="author">
    <slot />
  </article>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
