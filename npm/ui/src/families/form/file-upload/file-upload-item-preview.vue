<script setup lang="ts">
import { computed } from "vue";

import { fileUploadContext, fileUploadItemContext } from "./file-upload-context.ts";
import type {
  FileUploadItemPreviewSlotState,
  FileUploadPreviewState,
} from "./file-upload-types.ts";

const { alt = "" } = defineProps<{
  /**
   * Alternative text for the default `<img>`. Empty marks the preview decorative because
   * FileUploadItemName already names the file.
   *
   * @default ""
   */
  readonly alt?: string;
}>();

defineSlots<{
  /** Replaces the default `<img>`. Receives the object URL once created on the client. */
  default(props: FileUploadItemPreviewSlotState): unknown;
}>();

const context = fileUploadContext.use();
const item = fileUploadItemContext.use();
const state = computed<FileUploadPreviewState>(() => {
  if (!context.isPreviewable(item.state.value.file)) return "unsupported";
  return item.state.value.previewUrl === null ? "pending" : "ready";
});
const previewUrl = computed<string | null>(() => item.state.value.previewUrl);
const imageAttributes = computed<Record<string, string | undefined>>(() => ({
  src: previewUrl.value ?? undefined,
}));
const slotState = computed<FileUploadItemPreviewSlotState>(() => ({
  ...item.state.value,
  state: state.value,
}));
</script>

<template>
  <span data-vize-ui="file-upload-item-preview" part="item-preview" :data-state="state">
    <slot v-bind="slotState">
      <img
        v-if="previewUrl !== null"
        v-bind="imageAttributes"
        :alt
        data-vize-ui="file-upload-item-preview-image"
        part="item-preview-image"
      />
    </slot>
  </span>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
