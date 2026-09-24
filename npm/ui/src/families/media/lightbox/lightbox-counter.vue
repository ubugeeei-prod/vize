<script setup lang="ts">
import { computed, useTemplateRef } from "vue";

import { lightboxContext } from "./lightbox-context.ts";
import type { LightboxCounterSlotState, LightboxElementExpose } from "./lightbox-types.ts";

defineSlots<{
  /** Position text. Defaults to the `counter` message, e.g. "3 of 10". */
  default?(props: LightboxCounterSlotState): unknown;
}>();

const context = lightboxContext.use();
const element = useTemplateRef<HTMLDivElement>("element");
const position = computed<number>(() => context.index.value + 1);
const text = computed<string>(() =>
  context.messages.value.counter(position.value, context.count.value),
);
const slotState = computed<LightboxCounterSlotState>(() => ({
  count: context.count.value,
  position: position.value,
  text: text.value,
}));

const exposed = { element } satisfies Omit<LightboxElementExpose, "element"> & {
  readonly element: typeof element;
};

defineExpose(exposed);
</script>

<template>
  <div
    ref="element"
    aria-live="polite"
    aria-atomic="true"
    data-vize-ui="lightbox-counter"
    part="counter"
    :data-position="position"
  >
    <slot v-bind="slotState">{{ text }}</slot>
  </div>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
