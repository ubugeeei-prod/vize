<script setup lang="ts">
import { computed, useTemplateRef } from "vue";
import type { ComputedRef } from "vue";

import { imageContext } from "./image-context.ts";
import type { ImageFallbackExpose, ImagePartSlotState } from "./image-types.ts";

defineSlots<{
  /** Fallback content rendered after every candidate failed or none was safe. */
  default(props: ImagePartSlotState): unknown;
}>();

const context = imageContext.use();
const element = useTemplateRef<HTMLSpanElement>("element");
const visible = computed(() => context.status.value === "error");

type ImageFallbackSetupExpose = Omit<ImageFallbackExpose, "element" | "visible"> & {
  readonly element: typeof element;
  readonly visible: ComputedRef<boolean>;
};

const exposed = { element, visible } satisfies ImageFallbackSetupExpose;

defineExpose(exposed);
</script>

<template>
  <!-- eslint-disable vue/no-root-v-if -->
  <span
    v-if="visible"
    ref="element"
    data-vize-ui="image-fallback"
    part="fallback"
    :data-status="context.status.value"
  >
    <slot v-bind="context.slotState.value" />
  </span>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
