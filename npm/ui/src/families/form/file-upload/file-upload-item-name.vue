<script setup lang="ts">
import { fileUploadItemContext } from "./file-upload-context.ts";
import type { FileUploadItemSlotState } from "./file-upload-types.ts";

defineSlots<{
  /** Replaces the file name text. Receives per-file state. */
  default(props: FileUploadItemSlotState): unknown;
}>();

const item = fileUploadItemContext.use();
</script>

<template>
  <span
    data-vize-ui="file-upload-item-name"
    part="item-name"
    :title="item.state.value.relativePath ?? undefined"
  >
    <slot v-bind="item.state.value">{{ item.state.value.name }}</slot>
  </span>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
