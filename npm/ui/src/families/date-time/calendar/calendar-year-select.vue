<script setup lang="ts">
import { computed } from "vue";

import { calendarContext } from "./calendar-context.ts";

const {
  from = undefined,
  to = undefined,
  ariaLabel = "Year",
} = defineProps<{
  /** First listed year; defaults to the `min` year or ten years before the view. @default undefined */
  readonly from?: number;
  /** Last listed year; defaults to the `max` year or ten years after the view. @default undefined */
  readonly to?: number;
  /** Accessible name of the native select. @default "Year" */
  readonly ariaLabel?: string;
}>();

const emit = defineEmits<{
  /** Fired after the user picks a year. */
  change: [year: number, nativeEvent: Event];
}>();

const maximumYears = 400;
const context = calendarContext.use();
const years = computed(() => {
  const current = context.visibleStart.value?.year;
  if (current === undefined) return [];
  const first = Math.trunc(from ?? context.min.value?.year ?? current - 10);
  const last = Math.trunc(to ?? context.max.value?.year ?? current + 10);
  const start = Math.min(first, last, current);
  const end = Math.min(Math.max(first, last, current), start + maximumYears);
  return Array.from({ length: end - start + 1 }, (_, index) => {
    const year = start + index;
    return { year, label: context.formatters.value.year(year) };
  });
});
const disabled = computed(
  () => context.slotState.value.disabled || context.slotState.value.pending,
);

function onChange(event: Event): void {
  const start = context.visibleStart.value;
  if (!(event.currentTarget instanceof HTMLSelectElement) || !start) return;
  const year = Number(event.currentTarget.value);
  if (!Number.isInteger(year)) return;
  context.setVisibleMonth({ year, month: start.month });
  emit("change", year, event);
}
</script>

<template>
  <select
    :aria-label="ariaLabel"
    :aria-controls="context.id.value"
    :value="context.visibleStart.value ? String(context.visibleStart.value.year) : undefined"
    :disabled="disabled"
    data-vize-ui="calendar-year-select"
    part="year-select"
    :data-year="context.visibleStart.value?.year"
    @change="onChange"
  >
    <option
      v-for="option in years"
      :key="option.year"
      :value="String(option.year)"
      :selected="context.visibleStart.value?.year === option.year"
      data-vize-ui="calendar-year-option"
      part="year-option"
    >
      {{ option.label }}
    </option>
  </select>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
