<script setup lang="ts">
import { computed, ref } from "vue";

import type { ChatComposerProps } from "./types.ts";

const {
  label = "Message",
  submitLabel = "Send",
  disabled = false,
} = defineProps<ChatComposerProps>();

const emit = defineEmits<{
  /**
   * Fired when the user sends a non-empty message.
   *
   * Carries the trimmed text. Enter sends. Shift+Enter inserts a newline.
   */
  send: [text: string];
}>();

const draft = ref("");
const pending = computed(() => draft.value.trim().length === 0);

function onInput(event: Event): void {
  if (event.target instanceof HTMLTextAreaElement) draft.value = event.target.value;
}

function publish(): void {
  if (disabled || pending.value) return;
  emit("send", draft.value.trim());
  draft.value = "";
}

function onKeydown(event: KeyboardEvent): void {
  if (event.key !== "Enter" || event.shiftKey || event.isComposing) return;
  event.preventDefault();
  publish();
}
</script>

<template>
  <form role="form" data-vize-ui="chat-composer" aria-label="Composer" @submit.prevent="publish">
    <textarea
      :value="draft"
      rows="2"
      :aria-label="label"
      :disabled="disabled"
      @input="onInput"
      @keydown="onKeydown"
    />
    <button type="submit" :disabled="disabled || pending">{{ submitLabel }}</button>
  </form>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
