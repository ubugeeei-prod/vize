<script setup lang="ts">
import { useTemplateRef } from "vue";

import { toastRootContext, toastViewportContext } from "./toast-context.ts";
import type { ToastButtonExpose } from "./toast-types.ts";

const { altText } = defineProps<{
  /**
   * Required alternative that tells screen-reader users how to perform the
   * action without the toast, which may close before they reach it.
   *
   * @default required
   */
  readonly altText: string;
}>();

const emit = defineEmits<{
  /** Fired before the toast action runs. Call `preventDefault()` to keep the toast open. */
  click: [nativeEvent: MouseEvent];
}>();

defineSlots<{
  /** Action label. Falls back to the toast action `label`. */
  default(): unknown;
}>();

const context = toastRootContext.use();
const viewport = toastViewportContext.useOptional();
const element = useTemplateRef<HTMLButtonElement>("element");

function onClick(event: MouseEvent): void {
  emit("click", event);
  if (!event.defaultPrevented) context.toast.value.action?.onClick?.(event);
  if (event.defaultPrevented) return;
  if (context.dismiss("action")) viewport?.focus();
}

function focus(options?: FocusOptions): void {
  element.value?.focus(options);
}

type ToastActionSetupExpose = Omit<ToastButtonExpose, "element"> & {
  readonly element: typeof element;
};

const exposed = { element, focus } satisfies ToastActionSetupExpose;

defineExpose(exposed);
</script>

<template>
  <button
    ref="element"
    type="button"
    :data-alt-text="altText"
    data-vize-ui="toast-action"
    part="action"
    @click="onClick"
  >
    <slot>{{ context.toast.value.action?.label }}</slot>
  </button>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
