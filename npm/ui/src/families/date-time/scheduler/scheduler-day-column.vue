<script setup lang="ts">
import { onScopeDispose, useTemplateRef } from "vue";

import { schedulerContext } from "./scheduler-context.ts";
import SchedulerEvent from "./scheduler-event.vue";
import type { TimeGridPlacement } from "./scheduler-layout.ts";
import type { SchedulerDayColumnState } from "./scheduler-types.ts";

const { column } = defineProps<{
  /** Day and timed placements rendered by this column. @default undefined (required) */
  readonly column: SchedulerDayColumnState<unknown>;
}>();

defineSlots<{
  /** Event content for each placement; defaults to the event title. */
  event(props: TimeGridPlacement<unknown>): unknown;
}>();

const context = schedulerContext.use();
const element = useTemplateRef<HTMLDivElement>("element");
const target = context.registerDayTarget(() => column.day.date, "time", element);

onScopeDispose(() => target.dispose());
</script>

<template>
  <div
    ref="element"
    role="group"
    :aria-label="column.day.fullLabel"
    data-vize-ui="scheduler-day-column"
    part="day-column"
    :data-date="column.day.iso"
    :data-today="column.day.today ? 'true' : undefined"
    :data-drop-target="target.isOver.value ? 'true' : undefined"
  >
    <SchedulerEvent
      v-for="placement in column.placements"
      :key="placement.event.id"
      :event="placement.event"
      :placement
    >
      <slot name="event" v-bind="placement">{{ placement.event.title }}</slot>
    </SchedulerEvent>
  </div>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
