<script setup lang="ts">
import { computed } from "vue";

import { calendarContext } from "./calendar-context.ts";
import { compareDates, endOfMonth, startOfMonth } from "./plain-date.ts";

const { ariaLabel = "Month" } = defineProps<{
  /** Accessible name of the native select. @default "Month" */
  readonly ariaLabel?: string;
}>();

const emit = defineEmits<{
  /** Fired after the user picks a month, with the ISO month number. */
  change: [month: number, nativeEvent: Event];
}>();

const context = calendarContext.use();
const options = computed(() => {
  const start = context.visibleStart.value;
  const year = start?.year ?? 2000;
  return Array.from({ length: 12 }, (_, index) => {
    const month = index + 1;
    const min = context.min.value;
    const max = context.max.value;
    const disabled =
      (min !== null && compareDates(endOfMonth({ year, month }), min) < 0) ||
      (max !== null && compareDates(startOfMonth({ year, month }), max) > 0);
    return { month, label: context.formatters.value.monthName(month), disabled };
  });
});
const disabled = computed(
  () => context.slotState.value.disabled || context.slotState.value.pending,
);

function onChange(event: Event): void {
  const start = context.visibleStart.value;
  if (!(event.currentTarget instanceof HTMLSelectElement) || !start) return;
  const month = Number(event.currentTarget.value);
  if (!Number.isInteger(month) || month < 1 || month > 12) return;
  context.setVisibleMonth({ year: start.year, month });
  emit("change", month, event);
}
</script>

<template>
  <select
    :aria-label="ariaLabel"
    :aria-controls="context.id.value"
    :value="context.visibleStart.value ? String(context.visibleStart.value.month) : undefined"
    :disabled="disabled"
    data-vize-ui="calendar-month-select"
    part="month-select"
    :data-month="context.visibleStart.value?.month"
    @change="onChange"
  >
    <option
      v-for="option in options"
      :key="option.month"
      :value="String(option.month)"
      :selected="context.visibleStart.value?.month === option.month"
      :disabled="option.disabled"
      data-vize-ui="calendar-month-option"
      part="month-option"
    >
      {{ option.label }}
    </option>
  </select>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
