<!-- Week scheduler with typed events that applies drag and keyboard moves and resizes. -->
<script setup lang="ts">
import { ref } from "vue";

import { createPlainDateTime, SchedulerRoot } from "../scheduler.ts";
import type { SchedulerEventChange, SchedulerEventData } from "../scheduler.ts";

interface Booking {
  readonly room: string;
}

const events = ref<readonly SchedulerEventData<Booking>[]>([
  {
    id: "review",
    title: "Design review",
    data: { room: "Aurora" },
    start: createPlainDateTime(2026, 10, 22, 9, 0),
    end: createPlainDateTime(2026, 10, 22, 10, 30),
  },
  {
    id: "standup",
    title: "Standup",
    data: { room: "Cedar" },
    start: createPlainDateTime(2026, 10, 22, 9, 30),
    end: createPlainDateTime(2026, 10, 22, 9, 45),
  },
]);

function apply(change: SchedulerEventChange<Booking>): void {
  events.value = events.value.map((event) =>
    event.id === change.event.id ? { ...event, start: change.start, end: change.end } : event,
  );
}
</script>

<template>
  <SchedulerRoot
    :events
    aria-label="Room bookings"
    locale="en-US"
    :today="{ year: 2026, month: 10, day: 22 }"
    :day-start-hour="8"
    :day-end-hour="12"
    @event-move="apply"
    @event-resize="apply"
  />
</template>
