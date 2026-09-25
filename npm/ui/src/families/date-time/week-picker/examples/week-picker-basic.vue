<!-- Week picker with ISO week numbers, a fixed SSR-safe today, and ISO week submission. -->
<script setup lang="ts">
import { ref } from "vue";

import { WeekPicker, weekOf } from "../week-picker.ts";
import type { DateRange, PlainDate } from "../week-picker.ts";

function label(date: PlainDate): string {
  return `${date.year}-${String(date.month).padStart(2, "0")}-${String(date.day).padStart(2, "0")}`;
}

const sprint = ref<DateRange | null>(weekOf({ year: 2026, month: 10, day: 7 }, 1));
</script>

<template>
  <div>
    <WeekPicker
      v-model="sprint"
      name="sprint-week"
      locale="en-GB"
      aria-label="Sprint week"
      :today="{ year: 2026, month: 10, day: 1 }"
    />
    <output>
      {{ sprint ? `${label(sprint.start)} – ${label(sprint.end)}` : "No week" }}
    </output>
  </div>
</template>
