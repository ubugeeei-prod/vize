<script setup lang="ts">
import { useTemplateRef } from "vue";

import { imageCompareContext } from "./image-compare-context.ts";
import type {
  ImageComparePartExpose,
  ImageCompareSide,
  ImageCompareSlotState,
} from "./image-compare-types.ts";

const { side } = defineProps<{
  /** Which image this caption describes. @default required */
  readonly side: ImageCompareSide;
}>();

defineSlots<{
  /** Caption text such as "Before" or "After". Receives the divider state. */
  default(props: ImageCompareSlotState): unknown;
}>();

const context = imageCompareContext.use();
const element = useTemplateRef<HTMLSpanElement>("element");

const exposed = { element } satisfies {
  readonly element: typeof element;
} & Omit<ImageComparePartExpose, "element">;

defineExpose(exposed);
</script>

<template>
  <span
    ref="element"
    data-vize-ui="image-compare-label"
    part="label"
    :data-side="side"
    :data-orientation="context.orientation.value"
  >
    <slot v-bind="context.slotState.value" />
  </span>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
