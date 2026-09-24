<script setup lang="ts">
import { fileUploadItemContext } from "./file-upload-context.ts";
import type { FileUploadItemSlotState } from "./file-upload-types.ts";

defineSlots<{
  /** Replaces the formatted size text. Receives per-file state. */
  default(props: FileUploadItemSlotState): unknown;
}>();

const item = fileUploadItemContext.use();
</script>

<template>
  <data :value="item.state.value.size" data-vize-ui="file-upload-item-size" part="item-size">
    <slot v-bind="item.state.value">{{ item.state.value.formattedSize }}</slot>
  </data>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
