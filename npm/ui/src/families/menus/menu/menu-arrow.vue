<script setup lang="ts">
import { onMounted, onUpdated, useTemplateRef, watchEffect } from "vue";

import { positionerContext } from "../../overlays/positioner/positioner-context.ts";
import { menuLevelContext } from "./menu-context.ts";
import { alignOf, sideOf } from "./menu-dom.ts";
import type { MenuArrowExpose, MenuArrowSlotState } from "./menu-types.ts";

defineSlots<{
  /** Decorative arrow contents. Receives coordinates for optional custom drawing. */
  default(props: MenuArrowSlotState): unknown;
}>();

const level = menuLevelContext.use();
const positioner = positionerContext.use();
const element = useTemplateRef<HTMLDivElement>("element");

onMounted(() => positioner.setArrow(element.value));
onUpdated(() => positioner.setArrow(element.value));

watchEffect(
  () => {
    if (element.value) element.value.style.cssText = positioner.arrowStyle.value;
  },
  { flush: "sync" },
);

defineExpose({
  element,
  x: positioner.arrowX,
  y: positioner.arrowY,
} satisfies Omit<MenuArrowExpose, "element" | "x" | "y"> & {
  readonly element: typeof element;
  readonly x: typeof positioner.arrowX;
  readonly y: typeof positioner.arrowY;
});
</script>

<template>
  <div
    ref="element"
    aria-hidden="true"
    data-vize-ui="menu-arrow"
    part="arrow"
    :data-state="level.state.value"
    :data-side="sideOf(positioner.resolvedPlacement.value)"
    :data-align="alignOf(positioner.resolvedPlacement.value)"
  >
    <slot :x="positioner.arrowX.value" :y="positioner.arrowY.value" />
  </div>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
