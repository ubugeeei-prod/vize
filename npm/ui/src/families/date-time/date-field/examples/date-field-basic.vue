<!-- Segmented birthday input whose day, month, and year order follows the locale. -->
<script setup lang="ts">
import { ref, useId } from "vue";

import { DateField, type PlainDate } from "../date-field.ts";

const birthday = ref<PlainDate | null>({ year: 1990, month: 4, day: 12 });
const labelId = useId();
const max: PlainDate = { year: 2026, month: 1, day: 1 };

function formatDate(date: PlainDate): string {
  return `${date.year}-${String(date.month).padStart(2, "0")}-${String(date.day).padStart(2, "0")}`;
}
</script>

<template>
  <div>
    <span :id="labelId">Date of birth</span>
    <DateField v-model="birthday" name="birthday" locale="en-GB" :max :aria-labelledby="labelId" />
    <output>{{ birthday ? formatDate(birthday) : "No date entered" }}</output>
  </div>
</template>
