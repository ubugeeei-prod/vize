<script setup lang="ts">
import { useTemplateRef } from "vue";

import { toastRootContext } from "./toast-context.ts";
import type { ToastTextExpose } from "./toast-types.ts";

defineSlots<{
  /** Description contents. Falls back to the toast `description`. */
  default(): unknown;
}>();

const context = toastRootContext.use();
const element = useTemplateRef<HTMLDivElement>("element");

type ToastDescriptionSetupExpose = Omit<ToastTextExpose, "element"> & {
  readonly element: typeof element;
};

const exposed = { element } satisfies ToastDescriptionSetupExpose;

defineExpose(exposed);
</script>

<template>
  <div
    :id="context.descriptionId.value"
    ref="element"
    data-vize-ui="toast-description"
    part="description"
  >
    <slot>{{ context.toast.value.description }}</slot>
  </div>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
