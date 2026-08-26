<script setup lang="ts">
import { onMounted, onUpdated, useTemplateRef } from "vue";

import { positionerContext } from "./positioner-context.ts";

defineSlots<{
  /** Decorative arrow contents. Receives coordinates for optional custom drawing. */
  default(props: { readonly x: number | null; readonly y: number | null }): unknown;
}>();

const positioner = positionerContext.use();
const element = useTemplateRef<HTMLDivElement>("element");

onMounted(() => {
  positioner.setArrow(element.value);
});

onUpdated(() => {
  positioner.setArrow(element.value);
});

defineExpose({
  element,
  x: positioner.arrowX,
  y: positioner.arrowY,
});
</script>

<template>
  <div ref="element" data-vize-ui="positioner-arrow" :style="positioner.arrowStyle.value">
    <slot :x="positioner.arrowX.value" :y="positioner.arrowY.value" />
  </div>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
