<script setup lang="ts">
import { useTemplateRef } from "vue";

import { imageCompareContext } from "./image-compare-context.ts";
import type { ImageComparePartExpose, ImageCompareSlotState } from "./image-compare-types.ts";

defineSlots<{
  /** The before image or content. Receives the divider state. */
  default(props: ImageCompareSlotState): unknown;
}>();

const context = imageCompareContext.use();
const element = useTemplateRef<HTMLDivElement>("element");

const exposed = { element } satisfies {
  readonly element: typeof element;
} & Omit<ImageComparePartExpose, "element">;

defineExpose(exposed);
</script>

<template>
  <div
    ref="element"
    data-vize-ui="image-compare-before"
    part="before"
    data-side="before"
    :data-orientation="context.orientation.value"
    :data-state="context.state.value"
  >
    <slot v-bind="context.slotState.value" />
  </div>
</template>

<style scoped>
/* Headless by design. Clip one side with var(--vize-ui-image-compare-position). */
</style>
