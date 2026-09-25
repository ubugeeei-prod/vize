<!-- Single-date calendar bound with v-model, a fixed SSR-safe today, and a bookable range. -->
<script setup lang="ts">
import { ref } from "vue";

import {
  CalendarGrid,
  CalendarHeading,
  CalendarNext,
  CalendarPrev,
  CalendarRoot,
  createPlainDate,
  formatIsoDate,
  type PlainDate,
} from "../calendar.ts";

const today = createPlainDate(2026, 10, 1);
const selected = ref<PlainDate | null>(createPlainDate(2026, 10, 24));
const min = createPlainDate(2026, 10, 1);
const max = createPlainDate(2026, 12, 31);
</script>

<template>
  <div>
    <CalendarRoot v-model="selected" :today :min :max locale="en-US" name="event-date">
      <CalendarPrev />
      <CalendarHeading />
      <CalendarNext />
      <CalendarGrid />
    </CalendarRoot>
    <output>Selected: {{ selected ? formatIsoDate(selected) : "none" }}</output>
  </div>
</template>
