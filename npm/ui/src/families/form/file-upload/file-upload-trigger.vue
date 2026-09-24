<script setup lang="ts">
import { computed, onScopeDispose, useTemplateRef } from "vue";
import type { ComputedRef } from "vue";

import { fileUploadContext } from "./file-upload-context.ts";
import type { FileUploadActionExpose, FileUploadActionSlotState } from "./file-upload-types.ts";

const { ariaLabel = undefined, ariaDescribedby = undefined } = defineProps<{
  /**
   * Accessible name when the slot has no visible text.
   *
   * @default undefined
   */
  readonly ariaLabel?: string;

  /**
   * Space-separated ids of hints such as accepted types and size limits.
   *
   * @default undefined
   */
  readonly ariaDescribedby?: string;
}>();

const emit = defineEmits<{
  /** Fired before the picker opens. Call `preventDefault()` to keep it closed. */
  click: [nativeEvent: MouseEvent];
}>();

defineSlots<{
  /** Trigger contents. Receives availability and file count. */
  default(props: FileUploadActionSlotState): unknown;
}>();

const context = fileUploadContext.use();
const element = useTemplateRef<HTMLButtonElement>("element");
const disabled = computed(() => context.disabled.value);
const count = computed(() => context.files.value.length);
const slotState = computed<FileUploadActionSlotState>(() => ({
  count: count.value,
  disabled: disabled.value,
}));

function onClick(event: MouseEvent): void {
  if (disabled.value) return;
  emit("click", event);
  if (!event.defaultPrevented) context.openPicker();
}

function focus(options?: FocusOptions): void {
  element.value?.focus(options);
}

onScopeDispose(context.setFocusFallback(() => (disabled.value ? null : element.value)));

type FileUploadTriggerSetupExpose = Omit<
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
} satisfies FileUploadTriggerSetupExpose;

defineExpose(exposed);
</script>

<template>
  <button
    ref="element"
    type="button"
    :disabled="disabled"
    :aria-label="ariaLabel"
    :aria-describedby="ariaDescribedby"
    data-vize-ui="file-upload-trigger"
    part="trigger"
    :data-disabled="disabled ? 'true' : undefined"
    @click="onClick"
  >
    <slot v-bind="slotState" />
  </button>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
