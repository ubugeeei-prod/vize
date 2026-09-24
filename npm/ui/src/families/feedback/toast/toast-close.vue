<script setup lang="ts">
import { computed, useTemplateRef } from "vue";

import { toastRootContext, toastViewportContext } from "./toast-context.ts";
import type { ToastButtonExpose } from "./toast-types.ts";

const { ariaLabel = "Dismiss notification" } = defineProps<{
  /**
   * Accessible name of the close button.
   *
   * @default "Dismiss notification"
   */
  readonly ariaLabel?: string;
}>();

const emit = defineEmits<{
  /** Fired before closing. Call `preventDefault()` to keep the toast open. */
  click: [nativeEvent: MouseEvent];
}>();

defineSlots<{
  /** Close button contents. */
  default(): unknown;
}>();

const context = toastRootContext.use();
const viewport = toastViewportContext.useOptional();
const element = useTemplateRef<HTMLButtonElement>("element");
const disabled = computed(() => !context.toast.value.dismissible);

function onClick(event: MouseEvent): void {
  emit("click", event);
  if (event.defaultPrevented) return;
  if (context.dismiss("close")) viewport?.focus();
}

function focus(options?: FocusOptions): void {
  element.value?.focus(options);
}

type ToastCloseSetupExpose = Omit<ToastButtonExpose, "element"> & {
  readonly element: typeof element;
};

const exposed = { element, focus } satisfies ToastCloseSetupExpose;

defineExpose(exposed);
</script>

<template>
  <button
    ref="element"
    type="button"
    :disabled
    :aria-label="ariaLabel"
    data-vize-ui="toast-close"
    part="close"
    @click="onClick"
  >
    <slot>×</slot>
  </button>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
