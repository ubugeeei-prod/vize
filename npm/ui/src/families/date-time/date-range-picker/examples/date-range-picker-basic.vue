<!-- Hotel stay picker with check-in and check-out fields sharing one range calendar. -->
<script setup lang="ts">
import { ref } from "vue";

import {
  DateRangePicker,
  DateRangePickerCalendar,
  DateRangePickerContent,
  DateRangePickerField,
  DateRangePickerTrigger,
  type DateRange,
} from "../date-range-picker.ts";

const stay = ref<DateRange | null>({
  start: { year: 2026, month: 10, day: 23 },
  end: { year: 2026, month: 10, day: 25 },
});
</script>

<template>
  <DateRangePicker v-model="stay" start-name="checkIn" end-name="checkOut" locale="en-US">
    <DateRangePickerField boundary="start" aria-label="Check-in date" />
    <DateRangePickerField boundary="end" aria-label="Check-out date">
      <DateRangePickerTrigger aria-label="Choose stay dates">Calendar</DateRangePickerTrigger>
    </DateRangePickerField>
    <DateRangePickerContent aria-label="Stay calendar">
      <DateRangePickerCalendar :number-of-months="2" />
    </DateRangePickerContent>
  </DateRangePicker>
</template>
