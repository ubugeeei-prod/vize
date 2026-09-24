<script setup lang="ts">
import { calendarContext } from "./calendar-context.ts";
import type { CalendarHeadingSlotState } from "./calendar-types.ts";

defineSlots<{
  /** Heading content. Receives the localized heading and visible months. */
  default(props: CalendarHeadingSlotState): unknown;
}>();

const context = calendarContext.use();
</script>

<template>
  <h2
    :id="context.headingId.value"
    aria-live="polite"
    aria-atomic="true"
    data-vize-ui="calendar-heading"
    part="heading"
    :data-pending="context.slotState.value.pending ? 'true' : undefined"
  >
    <slot :heading="context.slotState.value.heading" :months="context.slotState.value.months">{{
      context.slotState.value.heading
    }}</slot>
  </h2>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
