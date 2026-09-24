<script setup lang="ts">
import { computed } from "vue";

import { timelineItemContext } from "./timeline-context.ts";
import type { TimelineItemSlotState } from "./timeline-types.ts";

defineSlots<{
  /** Item body such as a title and description. Receives the owning item's progress state. */
  default(props: TimelineItemSlotState): unknown;
}>();

const item = timelineItemContext.use();
const slotState = computed<TimelineItemSlotState>(() => ({
  index: item.index.value,
  last: item.last.value,
  status: item.status.value,
  value: item.value.value,
}));
</script>

<template>
  <div data-vize-ui="timeline-content" part="content" :data-state="item.status.value ?? undefined">
    <slot v-bind="slotState" />
  </div>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
