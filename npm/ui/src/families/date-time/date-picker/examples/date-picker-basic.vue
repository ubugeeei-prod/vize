<!-- Departure date picker combining a segmented field with a popover calendar. -->
<script setup lang="ts">
import { ref } from "vue";

import {
  DatePicker,
  DatePickerCalendar,
  DatePickerContent,
  DatePickerField,
  DatePickerTrigger,
  type PlainDate,
} from "../date-picker.ts";

const departure = ref<PlainDate | null>({ year: 2026, month: 10, day: 24 });
</script>

<template>
  <DatePicker v-model="departure" name="departure" locale="en-US">
    <DatePickerField aria-label="Departure date">
      <DatePickerTrigger aria-label="Choose departure date">Calendar</DatePickerTrigger>
    </DatePickerField>
    <DatePickerContent aria-label="Departure calendar">
      <DatePickerCalendar />
    </DatePickerContent>
  </DatePicker>
</template>
