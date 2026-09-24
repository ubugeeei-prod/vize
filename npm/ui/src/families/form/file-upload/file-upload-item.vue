<script setup lang="ts">
import { computed } from "vue";

import { fileUploadContext, fileUploadItemContext } from "./file-upload-context.ts";
import type { FileUploadItemSlotState } from "./file-upload-types.ts";

const { file } = defineProps<{
  /** File described by this item and its parts. @default required */
  readonly file: File;
}>();

defineSlots<{
  /** Item contents. Receives per-file name, size, type, path, and preview state. */
  default(props: FileUploadItemSlotState): unknown;
}>();

const context = fileUploadContext.use();
const state = computed(() => context.getItemState(file));
const index = computed<number>(() => state.value.index);
const type = computed<string | undefined>(() => state.value.type || undefined);
const disabled = computed<boolean>(() => state.value.disabled);
fileUploadItemContext.provide({ state });
</script>

<template>
  <li
    data-vize-ui="file-upload-item"
    part="item"
    :data-index="index"
    :data-type="type"
    :data-disabled="disabled ? 'true' : undefined"
  >
    <slot v-bind="state" />
  </li>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
