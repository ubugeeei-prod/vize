<script setup lang="ts">
import { useTemplateRef } from "vue";
import type { ComputedRef } from "vue";

import { panZoomContext } from "./pan-zoom-context.ts";
import type { PanZoomSlotState, PanZoomStatusExpose } from "./pan-zoom-types.ts";

defineSlots<{
  /**
   * Announcement content. Defaults to the localized settled zoom level, updated
   * when a gesture ends rather than on every frame.
   */
  default(props: PanZoomSlotState & { readonly text: string }): unknown;
}>();

const context = panZoomContext.use();
const element = useTemplateRef<HTMLDivElement>("element");

type PanZoomStatusSetupExpose = Omit<PanZoomStatusExpose, "element" | "text"> & {
  readonly element: typeof element;
  readonly text: ComputedRef<string>;
};

const exposed = { element, text: context.statusText } satisfies PanZoomStatusSetupExpose;

defineExpose(exposed);
</script>

<template>
  <div
    ref="element"
    role="status"
    aria-live="polite"
    aria-atomic="true"
    data-vize-ui="pan-zoom-status"
    part="status"
  >
    <slot v-bind="context.slotState.value" :text="context.statusText.value">{{
      context.statusText.value
    }}</slot>
  </div>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
