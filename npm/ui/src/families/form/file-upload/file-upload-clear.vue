<script setup lang="ts">
import { computed, nextTick, useTemplateRef } from "vue";
import type { ComputedRef } from "vue";

import { fileUploadContext } from "./file-upload-context.ts";
import type { FileUploadActionExpose, FileUploadActionSlotState } from "./file-upload-types.ts";

const { ariaLabel = undefined } = defineProps<{
  /**
   * Accessible name when the slot has no visible text.
   *
   * @default undefined
   */
  readonly ariaLabel?: string;
}>();

const emit = defineEmits<{
  /** Fired before files are cleared. Call `preventDefault()` to keep them. */
  click: [nativeEvent: MouseEvent];
}>();

defineSlots<{
  /** Button contents. Receives availability and file count. */
  default(props: FileUploadActionSlotState): unknown;
}>();

const context = fileUploadContext.use();
const element = useTemplateRef<HTMLButtonElement>("element");
const count = computed(() => context.files.value.length);
const disabled = computed(() => context.disabled.value || count.value === 0);
const slotState = computed<FileUploadActionSlotState>(() => ({
  count: count.value,
  disabled: disabled.value,
}));

function onClick(event: MouseEvent): void {
  if (disabled.value) return;
  emit("click", event);
  if (event.defaultPrevented || !context.clear()) return;
  // The button becomes disabled with an empty list, so keep focus inside the upload.
  void nextTick(context.focusFallback);
}

function focus(options?: FocusOptions): void {
  element.value?.focus(options);
}

type FileUploadClearSetupExpose = Omit<
  FileUploadActionExpose,
  keyof FileUploadActionSlotState | "element"
> & {
  readonly count: ComputedRef<number>;
  readonly disabled: ComputedRef<boolean>;
  readonly element: typeof element;
};

const exposed = {
  count,
  disabled,
  element,
  focus,
} satisfies FileUploadClearSetupExpose;

defineExpose(exposed);
</script>

<template>
  <button
    ref="element"
    type="button"
    :disabled="disabled"
    :aria-label="ariaLabel"
    data-vize-ui="file-upload-clear"
    part="clear"
    :data-disabled="disabled ? 'true' : undefined"
    @click="onClick"
  >
    <slot v-bind="slotState" />
  </button>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
