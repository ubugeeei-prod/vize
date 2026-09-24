<script setup lang="ts">
import { onMounted, onUnmounted, useTemplateRef } from "vue";

import CalendarRoot from "../calendar/calendar-root.vue";
import type { CalendarWeekdayFormat } from "../calendar/calendar-locale.ts";
import type { CalendarRootExpose, CalendarSlotState } from "../calendar/calendar-types.ts";
import type { PlainDate, Weekday } from "../calendar/plain-date.ts";
import { datePickerContext } from "./date-picker-context.ts";

const {
  numberOfMonths = 1,
  pagedNavigation = false,
  fixedWeeks = false,
  weekStartsOn = undefined,
  weekdayFormat = "short",
  calendar = undefined,
  numberingSystem = undefined,
  ariaLabel = undefined,
} = defineProps<{
  /** Number of consecutive months rendered. @default 1 */
  readonly numberOfMonths?: number;
  /** Move month controls by `numberOfMonths`. @default false */
  readonly pagedNavigation?: boolean;
  /** Always render six week rows per month. @default false */
  readonly fixedWeeks?: boolean;
  /** First day of week (`0` = Sunday); defaults to the locale preference. @default undefined */
  readonly weekStartsOn?: Weekday;
  /** Weekday column label width. @default "short" */
  readonly weekdayFormat?: CalendarWeekdayFormat;
  /** Intl calendar used for display labels only. @default undefined */
  readonly calendar?: string;
  /** Intl numbering system for labels. @default undefined */
  readonly numberingSystem?: string;
  /** Accessible name for the calendar group. @default undefined */
  readonly ariaLabel?: string;
}>();

defineSlots<{
  /** Calendar composition; defaults to a header plus one grid per month. */
  default(props: CalendarSlotState): unknown;
}>();

const context = datePickerContext.use();
const calendarRoot = useTemplateRef<CalendarRootExpose>("calendarRoot");
let unregister: (() => void) | null = null;

function focusTarget(): HTMLElement | null {
  const element = calendarRoot.value?.root?.querySelector(
    "[data-vize-ui='calendar-day'][tabindex='0']",
  );
  return element instanceof HTMLElement ? element : null;
}

onMounted(() => {
  unregister = context.registerFocusTarget(focusTarget);
});

onUnmounted(() => {
  unregister?.();
});

function onSelect(value: PlainDate, event: Event): void {
  context.setValue(value, event);
  if (context.options.value.closeOnSelect) context.setOpen(false, event);
}
</script>

<template>
  <CalendarRoot
    ref="calendarRoot"
    :model-value="context.value.value"
    :min="context.options.value.min"
    :max="context.options.value.max"
    :is-date-unavailable="context.options.value.isDateUnavailable"
    :locale="context.options.value.locale"
    :dir="context.options.value.dir"
    :today="context.options.value.today"
    :now="context.options.value.now"
    :time-zone="context.options.value.timeZone"
    :disabled="context.disabled.value"
    :read-only="context.readOnly.value"
    :number-of-months
    :paged-navigation
    :fixed-weeks
    :week-starts-on
    :weekday-format
    :calendar
    :numbering-system
    :aria-label
    data-picker-part="calendar"
    @select="onSelect"
  >
    <template v-if="$slots.default" #default="state: CalendarSlotState"
      ><slot v-bind="state"
    /></template>
  </CalendarRoot>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
