<script setup lang="ts">
import { onMounted, onUpdated, useTemplateRef, watchEffect } from "vue";

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

// Arrow geometry is measurement-driven output, not consumer-owned styling;
// apply it imperatively so templates stay free of `:style` bindings.
watchEffect(
  () => {
    if (element.value) {
      element.value.style.cssText = positioner.arrowStyle.value;
    }
  },
  // Sync flush publishes arrow geometry as soon as the element ref lands.
  { flush: "sync" },
);

defineExpose({
  element,
  x: positioner.arrowX,
  y: positioner.arrowY,
});
</script>

<template>
  <div ref="element" data-vize-ui="positioner-arrow">
    <slot :x="positioner.arrowX.value" :y="positioner.arrowY.value" />
  </div>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
