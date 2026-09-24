<script setup lang="ts">
import { computed } from "vue";

import { timelineItemContext } from "./timeline-context.ts";
import type { TimelineItemSlotState } from "./timeline-types.ts";

defineSlots<{
  /** Optional decorative content. Receives the owning item's progress state. */
  default?(props: TimelineItemSlotState): unknown;
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
  <span
    aria-hidden="true"
    data-vize-ui="timeline-indicator"
    part="indicator"
    :data-state="item.status.value ?? undefined"
    :data-last="item.last.value ? 'true' : undefined"
  >
    <slot v-bind="slotState" />
  </span>
</template>

<style scoped>
/* Headless by design. Decorative parts stay out of the accessibility tree. */
</style>
