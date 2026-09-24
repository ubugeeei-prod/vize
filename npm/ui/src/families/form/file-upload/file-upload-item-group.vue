<script setup lang="ts">
import { computed } from "vue";

import { fileUploadContext } from "./file-upload-context.ts";
import type { FileUploadItemGroupSlotState, FileUploadState } from "./file-upload-types.ts";

const { ariaLabel = undefined, ariaLabelledby = undefined } = defineProps<{
  /**
   * Accessible name for the file list.
   *
   * @default undefined
   */
  readonly ariaLabel?: string;

  /**
   * Space-separated ids that label the file list.
   *
   * @default undefined
   */
  readonly ariaLabelledby?: string;
}>();

defineSlots<{
  /** List contents, usually one FileUploadItem per entry of `items`. */
  default(props: FileUploadItemGroupSlotState): unknown;
}>();

const context = fileUploadContext.use();
const state = computed<FileUploadState>(() => context.state.value);
const count = computed<number>(() => context.files.value.length);
const slotState = computed<FileUploadItemGroupSlotState>(() => ({
  files: context.files.value,
  items: context.files.value.map((file) => context.getItemState(file)),
  state: context.state.value,
}));
</script>

<template>
  <ul
    :aria-label="ariaLabel"
    :aria-labelledby="ariaLabelledby"
    data-vize-ui="file-upload-item-group"
    part="item-group"
    :data-state="state"
    :data-count="count"
  >
    <slot v-bind="slotState" />
  </ul>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
