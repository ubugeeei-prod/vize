<script setup lang="ts">
import { onMounted, onUpdated, useTemplateRef, watchEffect } from "vue";

import { popoverContext } from "./popover-context.ts";
import type { PopoverArrowExpose, PopoverArrowSlotState } from "./popover-types.ts";
import { positionerContext } from "../positioner/positioner-context.ts";
import type { Placement, PlacementAlign, PlacementSide } from "../positioner/positioner.ts";

defineSlots<{
  /** Decorative arrow contents. Receives coordinates for optional custom drawing. */
  default(props: PopoverArrowSlotState): unknown;
}>();

const context = popoverContext.use();
const positioner = positionerContext.use();
const element = useTemplateRef<HTMLDivElement>("element");

onMounted(() => {
  positioner.setArrow(element.value);
});

onUpdated(() => {
  positioner.setArrow(element.value);
});

watchEffect(
  () => {
    if (element.value) {
      element.value.style.cssText = positioner.arrowStyle.value;
    }
  },
  { flush: "sync" },
);

function sideFromPlacement(value: Placement): PlacementSide {
  return value.split("-", 1)[0] as PlacementSide;
}

function alignFromPlacement(value: Placement): PlacementAlign {
  const [, align] = value.split("-");
  return (align ?? "center") as PlacementAlign;
}

type PopoverArrowSetupExpose = Omit<PopoverArrowExpose, "element" | "x" | "y"> & {
  readonly element: typeof element;
  readonly x: typeof positioner.arrowX;
  readonly y: typeof positioner.arrowY;
};

const exposed = {
  element,
  x: positioner.arrowX,
  y: positioner.arrowY,
} satisfies PopoverArrowSetupExpose;

defineExpose(exposed);
</script>

<template>
  <div
    ref="element"
    data-vize-ui="popover-arrow"
    part="arrow"
    :data-state="context.state.value"
    :data-side="sideFromPlacement(positioner.resolvedPlacement.value)"
    :data-align="alignFromPlacement(positioner.resolvedPlacement.value)"
  >
    <slot :x="positioner.arrowX.value" :y="positioner.arrowY.value" />
  </div>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
