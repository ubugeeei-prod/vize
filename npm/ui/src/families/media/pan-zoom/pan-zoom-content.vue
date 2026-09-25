<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, useTemplateRef } from "vue";

import { panZoomContext } from "./pan-zoom-context.ts";
import { transformToCss } from "./pan-zoom-transform.ts";
import type { PanZoomContentExpose, PanZoomSlotState } from "./pan-zoom-types.ts";

defineSlots<{
  /** Transformed content. Receives the current transform state. */
  default(props: PanZoomSlotState): unknown;
}>();

const context = panZoomContext.use();
const element = useTemplateRef<HTMLDivElement>("element");
// The transform is positioning mechanics rather than styling, so it ships as an
// inline style; the same values are published as custom properties.
const style = computed(() => {
  const { x, y, scale } = context.transform.value;
  return {
    "--vize-ui-pan-zoom-x": `${x}px`,
    "--vize-ui-pan-zoom-y": `${y}px`,
    "--vize-ui-pan-zoom-scale": String(scale),
    transform: transformToCss(context.transform.value),
    "transform-origin": "0 0",
  };
});

onMounted(() => context.setContentElement(element.value));
onBeforeUnmount(() => context.setContentElement(null));

type PanZoomContentSetupExpose = Omit<PanZoomContentExpose, "element"> & {
  readonly element: typeof element;
};

const exposed = { element } satisfies PanZoomContentSetupExpose;

defineExpose(exposed);
</script>

<template>
  <div
    ref="element"
    data-vize-ui="pan-zoom-content"
    part="content"
    :data-state="context.state.value"
    :style
  >
    <slot v-bind="context.slotState.value" />
  </div>
</template>

<style scoped>
/* Headless by design. Only the transform mechanics are applied inline. */
</style>
