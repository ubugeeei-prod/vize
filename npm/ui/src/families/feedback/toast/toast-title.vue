<script setup lang="ts">
import { useTemplateRef } from "vue";

import { toastRootContext } from "./toast-context.ts";
import type { ToastTextExpose } from "./toast-types.ts";

defineSlots<{
  /** Title contents. Falls back to the toast `title`. */
  default(): unknown;
}>();

const context = toastRootContext.use();
const element = useTemplateRef<HTMLDivElement>("element");

type ToastTitleSetupExpose = Omit<ToastTextExpose, "element"> & {
  readonly element: typeof element;
};

const exposed = { element } satisfies ToastTitleSetupExpose;

defineExpose(exposed);
</script>

<template>
  <div :id="context.titleId.value" ref="element" data-vize-ui="toast-title" part="title">
    <slot>{{ context.toast.value.title }}</slot>
  </div>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
