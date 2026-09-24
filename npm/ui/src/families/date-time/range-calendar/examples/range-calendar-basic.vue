<!-- Controlled two-click date range calendar with a fixed, SSR-safe "today". -->
<script setup lang="ts">
import { ref } from "vue";

import { RangeCalendar } from "../range-calendar.ts";
import type { DateRange, PlainDate } from "../range-calendar.ts";

const today: PlainDate = { year: 2026, month: 9, day: 25 };
const stay = ref<DateRange | null>({
  start: { year: 2026, month: 9, day: 8 },
  end: { year: 2026, month: 9, day: 11 },
});

function format(date: PlainDate): string {
  return `${date.year}-${String(date.month).padStart(2, "0")}-${String(date.day).padStart(2, "0")}`;
}
</script>

<template>
  <div>
    <RangeCalendar
      v-model="stay"
      :today
      locale="en-US"
      aria-label="Hotel stay"
      start-name="check-in"
      end-name="check-out"
    />
    <output>{{
      stay ? `${format(stay.start)} → ${format(stay.end)}` : "No dates selected"
    }}</output>
  </div>
</template>
