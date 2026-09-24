<script setup lang="ts">
import { computed, onMounted, onUpdated, useTemplateRef, watchEffect } from "vue";

import { positionerContext } from "../positioner/positioner-context.ts";
import { tourContext } from "./tour-context.ts";
import { tourAlignFromPlacement, tourSideFromPlacement } from "./tour-state.ts";
import type { TourArrowSlotState, TourContentPlacement } from "./tour-types.ts";

defineSlots<{
  /** Decorative arrow contents. Receives coordinates for optional custom drawing. */
  default(props: TourArrowSlotState): unknown;
}>();

const context = tourContext.use();
const positioner = positionerContext.use();
const element = useTemplateRef<HTMLDivElement>("element");
const anchored = computed(() => context.targetState.value === "resolved");
const placement = computed<TourContentPlacement>(() =>
  anchored.value ? positioner.resolvedPlacement.value : "center",
);

onMounted(() => {
  positioner.setArrow(element.value);
});

onUpdated(() => {
  positioner.setArrow(element.value);
});

// Arrow geometry is measurement output applied imperatively, like PopoverArrow.
watchEffect(
  () => {
    if (element.value)
      element.value.style.cssText = anchored.value ? positioner.arrowStyle.value : "";
  },
  { flush: "sync" },
);

defineExpose({
  element,
  x: positioner.arrowX,
  y: positioner.arrowY,
});
</script>

<template>
  <div
    ref="element"
    aria-hidden="true"
    data-vize-ui="tour-arrow"
    part="arrow"
    :hidden="anchored ? undefined : true"
    :data-state="context.state.value"
    :data-side="tourSideFromPlacement(placement)"
    :data-align="tourAlignFromPlacement(placement)"
  >
    <slot :x="positioner.arrowX.value" :y="positioner.arrowY.value" />
  </div>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
