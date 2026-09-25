<!-- Listbox of half-hour appointment slots with an unavailable lunch break. -->
<script setup lang="ts">
import { ref } from "vue";

import { TimePicker } from "../time-picker.ts";
import type { PlainTime } from "../time-picker.ts";

const slot = ref<PlainTime | null>({ hour: 10, minute: 0, second: 0 });

function label(time: PlainTime): string {
  return `${String(time.hour).padStart(2, "0")}:${String(time.minute).padStart(2, "0")}`;
}

function isLunch(time: PlainTime): boolean {
  return time.hour === 12;
}
</script>

<template>
  <div>
    <TimePicker
      v-model="slot"
      name="appointment"
      locale="en-US"
      aria-label="Appointment time"
      :min="{ hour: 9, minute: 0, second: 0 }"
      :max="{ hour: 17, minute: 0, second: 0 }"
      :step="30"
      :is-time-unavailable="isLunch"
    />
    <output>{{ slot ? label(slot) : "No slot" }}</output>
  </div>
</template>
