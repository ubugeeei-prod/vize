<script setup lang="ts">
import { computed, onMounted, useTemplateRef, watch } from "vue";

import { useSizeObserver } from "../../interaction/measure/measure-runtime.ts";
import { marqueeContext } from "./marquee-context.ts";
import type { MarqueeContentExpose, MarqueeSlotState } from "./marquee-types.ts";

defineSlots<{
  /**
   * Content rendered once per copy. Only the first copy is exposed to assistive
   * technology and sequential focus; the rest are `aria-hidden` and `inert`.
   * Avoid ids inside the slot, since copies duplicate them.
   */
  default(props: MarqueeSlotState & { readonly copy: number }): unknown;
}>();

const context = marqueeContext.use();
const element = useTemplateRef<HTMLDivElement>("element");
const copyIndexes = computed<readonly number[]>(() =>
  Array.from({ length: context.copies.value }, (_, index) => index),
);

function axisStart(rect: DOMRect): number {
  return context.orientation.value === "vertical" ? rect.top : rect.left;
}

function axisLength(rect: DOMRect): number {
  return context.orientation.value === "vertical" ? rect.height : rect.width;
}

function measure(): void {
  if (element.value === null || context.element.value === null) return;
  const first = element.value.children.item(0);
  const second = element.value.children.item(1);
  if (first === null) return;
  const firstRect = first.getBoundingClientRect();
  const distance =
    second === null
      ? axisLength(firstRect)
      : Math.abs(axisStart(second.getBoundingClientRect()) - axisStart(firstRect));
  context.setMeasurement({
    distance,
    viewport: axisLength(context.element.value.getBoundingClientRect()),
  });
}

const observer = useSizeObserver({ onResize: measure });

onMounted(() => {
  if (element.value === null) return;
  const first = element.value.children.item(0);
  if (first !== null) observer.observe(first);
  if (context.element.value !== null) observer.observe(context.element.value);
  measure();
});

watch(context.orientation, measure, { flush: "post" });

const exposed = { element, measure } satisfies Omit<MarqueeContentExpose, "element"> & {
  readonly element: typeof element;
};

defineExpose(exposed);
</script>

<template>
  <div
    ref="element"
    data-vize-ui="marquee-track"
    part="track"
    :data-state="context.state.value"
    :data-orientation="context.orientation.value"
  >
    <div
      v-for="copy in copyIndexes as readonly number[]"
      :key="copy"
      data-vize-ui="marquee-content"
      part="content"
      :data-copy="copy"
      :aria-hidden="copy > 0 ? 'true' : undefined"
      :inert="copy > 0 ? true : undefined"
    >
      <slot v-bind="context.slotState.value" :copy />
    </div>
  </div>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
