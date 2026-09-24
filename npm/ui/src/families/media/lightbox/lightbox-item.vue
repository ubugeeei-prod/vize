<script setup lang="ts">
import { computed, useTemplateRef } from "vue";
import type { ComputedRef } from "vue";

import { lightboxContext } from "./lightbox-context.ts";
import type { LightboxCounterSlotState, LightboxItemExpose } from "./lightbox-types.ts";

defineSlots<{
  /** Current media. Render the root slot's `item` here. */
  default(props: LightboxCounterSlotState): unknown;
}>();

const context = lightboxContext.use();
const element = useTemplateRef<HTMLDivElement>("element");
const itemId = computed<string>(() => context.getItemId(context.index.value));
const position = computed<number>(() => context.index.value + 1);
const text = computed<string>(() =>
  context.messages.value.counter(position.value, context.count.value),
);
const slotState = computed<LightboxCounterSlotState>(() => ({
  count: context.count.value,
  position: position.value,
  text: text.value,
}));

type LightboxItemSetupExpose = Omit<LightboxItemExpose, "element" | "id"> & {
  readonly element: typeof element;
  readonly id: ComputedRef<string>;
};

const exposed = { element, id: itemId } satisfies LightboxItemSetupExpose;

defineExpose(exposed);
</script>

<template>
  <div
    :id="itemId"
    ref="element"
    role="group"
    :aria-roledescription="context.messages.value.slide"
    :aria-label="text"
    data-vize-ui="lightbox-item"
    part="item"
    :data-index="context.index.value"
  >
    <slot v-bind="slotState" />
  </div>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
