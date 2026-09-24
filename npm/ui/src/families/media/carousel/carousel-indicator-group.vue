<script setup lang="ts">
import { useTemplateRef } from "vue";

import { carouselContext } from "./carousel-context.ts";
import type { CarouselIndicatorGroupExpose, CarouselSlotState } from "./carousel-types.ts";

const { ariaLabel = undefined, ariaLabelledby = undefined } = defineProps<{
  /**
   * Accessible name of the slide picker, e.g. "Choose slide".
   *
   * @default undefined
   */
  readonly ariaLabel?: string;

  /**
   * Space-separated ids that label the slide picker.
   *
   * @default undefined
   */
  readonly ariaLabelledby?: string;
}>();

defineSlots<{
  /** CarouselIndicator children. Receives the carousel state. */
  default(props: CarouselSlotState): unknown;
}>();

const context = carouselContext.use();
const element = useTemplateRef<HTMLDivElement>("element");

type CarouselIndicatorGroupSetupExpose = Omit<CarouselIndicatorGroupExpose, "element"> & {
  readonly element: typeof element;
};

const exposed = { element } satisfies CarouselIndicatorGroupSetupExpose;

defineExpose(exposed);
</script>

<template>
  <div
    ref="element"
    role="tablist"
    :aria-label="ariaLabel"
    :aria-labelledby="ariaLabelledby"
    :aria-orientation="context.orientation.value"
    data-vize-ui="carousel-indicator-group"
    part="indicator-group"
    :data-orientation="context.orientation.value"
  >
    <slot v-bind="context.slotState.value" />
  </div>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
