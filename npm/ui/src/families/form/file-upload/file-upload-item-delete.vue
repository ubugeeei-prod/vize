<script setup lang="ts">
import { computed, nextTick, useTemplateRef } from "vue";

import { fileUploadContext, fileUploadItemContext } from "./file-upload-context.ts";
import type { FileUploadItemSlotState } from "./file-upload-types.ts";

const { ariaLabel = undefined } = defineProps<{
  /**
   * Accessible name when the slot has no visible text, for example `Remove report.pdf`.
   *
   * @default undefined
   */
  readonly ariaLabel?: string;
}>();

const emit = defineEmits<{
  /** Fired before the file is removed. Call `preventDefault()` to keep it. */
  click: [nativeEvent: MouseEvent];
}>();

defineSlots<{
  /** Button contents. Receives per-file state. */
  default(props: FileUploadItemSlotState): unknown;
}>();

const context = fileUploadContext.use();
const item = fileUploadItemContext.use();
const element = useTemplateRef<HTMLButtonElement>("element");
const disabled = computed(() => item.state.value.disabled);
const DELETE_SELECTOR = '[data-vize-ui="file-upload-item-delete"]:not([disabled])';

function neighbourDelete(): HTMLElement | null {
  const listItem = element.value?.closest('[data-vize-ui="file-upload-item"]');
  if (!listItem) return null;
  for (const sibling of [listItem.nextElementSibling, listItem.previousElementSibling]) {
    const candidate = sibling?.querySelector(DELETE_SELECTOR);
    if (candidate instanceof HTMLElement) return candidate;
  }
  return null;
}

function onClick(event: MouseEvent): void {
  if (disabled.value) return;
  emit("click", event);
  if (event.defaultPrevented) return;
  const next = neighbourDelete();
  if (!context.removeFile(item.state.value.file)) return;
  // Keep focus on the next logical control instead of dropping it to the document body.
  void nextTick(() => {
    if (next?.isConnected) next.focus();
    else context.focusFallback();
  });
}
</script>

<template>
  <button
    ref="element"
    type="button"
    :disabled="disabled"
    :aria-label="ariaLabel"
    data-vize-ui="file-upload-item-delete"
    part="item-delete"
    @click="onClick"
  >
    <slot v-bind="item.state.value" />
  </button>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
