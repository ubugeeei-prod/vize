<script setup lang="ts">
import { onUnmounted, useTemplateRef, watch } from "vue";

import { selectContext } from "./select-context.ts";

defineSlots<{
  /** Scrollable options region. Scroll buttons and `SelectVirtualizer` observe this element. */
  default(): unknown;
}>();

const context = selectContext.use();
const element = useTemplateRef<HTMLDivElement>("element");

watch(
  element,
  (next, previous) => {
    if (previous !== null && context.viewportElement.value === previous) {
      context.viewportElement.value = null;
    }
    if (next !== null) context.viewportElement.value = next;
  },
  { flush: "post" },
);

onUnmounted(() => {
  if (context.viewportElement.value === element.value) context.viewportElement.value = null;
});

defineExpose({ element });
</script>

<template>
  <div
    ref="element"
    role="presentation"
    :data-vize-ui="`${context.partPrefix}-viewport`"
    part="viewport"
  >
    <slot />
  </div>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
