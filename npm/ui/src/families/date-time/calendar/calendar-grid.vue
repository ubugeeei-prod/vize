<script setup lang="ts">
import { computed } from "vue";

import { deriveDeterministicId } from "../../foundations/id/deterministic-id.ts";
import { calendarContext } from "./calendar-context.ts";
import type {
  CalendarDayState,
  CalendarMonthState,
  CalendarWeekdaySlotState,
} from "./calendar-types.ts";

const { monthIndex = 0 } = defineProps<{
  /** Zero-based index of the visible month this grid renders. @default 0 */
  readonly monthIndex?: number;
}>();

defineSlots<{
  /** Weekday column header content. Receives the weekday, its labels, and column index. */
  weekday(props: CalendarWeekdaySlotState): unknown;
  /** Day button content. Receives the complete day state; defaults to the localized day number. */
  day(props: CalendarDayState): unknown;
}>();

const context = calendarContext.use();
const month = computed<CalendarMonthState | null>(
  () => context.slotState.value.months[monthIndex] ?? null,
);
const gridId = computed(() => deriveDeterministicId(context.id.value, `grid-${monthIndex}`));
const monthKey = computed(() =>
  month.value
    ? `${String(month.value.year).padStart(4, "0")}-${String(month.value.month).padStart(2, "0")}`
    : undefined,
);

function weekdaySlotState(
  weekday: CalendarWeekdaySlotState | Omit<CalendarWeekdaySlotState, "column">,
  column: number,
): CalendarWeekdaySlotState {
  return { weekday: weekday.weekday, label: weekday.label, longLabel: weekday.longLabel, column };
}

function onDayClick(day: CalendarDayState, event: MouseEvent): void {
  context.onDayClick(day.date, event);
}

function onDayKeydown(day: CalendarDayState, event: KeyboardEvent): void {
  context.onDayKeydown(day.date, event);
}

function onDayPointerEnter(day: CalendarDayState): void {
  context.onDayHover(day.date);
}

function onGridPointerLeave(): void {
  context.onDayHover(null);
}

function selectedAttribute(day: CalendarDayState): "true" | "false" | undefined {
  if (day.outsideMonth) return undefined;
  return day.selected || (day.inRange && !day.preview) ? "true" : "false";
}
</script>

<template>
  <table
    :id="gridId"
    role="grid"
    :aria-label="month?.label"
    :aria-readonly="context.slotState.value.readOnly ? 'true' : undefined"
    :aria-disabled="context.slotState.value.disabled ? 'true' : undefined"
    :aria-multiselectable="context.slotState.value.mode === 'range' ? 'true' : undefined"
    data-vize-ui="calendar-grid"
    part="grid"
    :data-month-index="monthIndex"
    :data-month="monthKey"
    :data-pending="month === null ? 'true' : undefined"
  >
    <thead data-vize-ui="calendar-grid-head" part="grid-head">
      <tr data-vize-ui="calendar-weekdays" part="weekdays">
        <th
          v-for="(weekday, column) in context.slotState.value.weekdays"
          :key="weekday.weekday"
          scope="col"
          :abbr="weekday.longLabel"
          data-vize-ui="calendar-weekday"
          part="weekday"
          :data-weekday="weekday.weekday"
        >
          <slot name="weekday" v-bind="weekdaySlotState(weekday, column)">{{ weekday.label }}</slot>
        </th>
      </tr>
    </thead>
    <tbody data-vize-ui="calendar-grid-body" part="grid-body" @pointerleave="onGridPointerLeave">
      <tr
        v-for="(week, row) in month?.weeks ?? []"
        :key="week[0]?.iso ?? row"
        data-vize-ui="calendar-week"
        part="week"
        :data-week="row"
      >
        <td
          v-for="day in week"
          :key="day.iso"
          :aria-selected="selectedAttribute(day)"
          :aria-disabled="day.disabled || day.unavailable || day.outsideMonth ? 'true' : undefined"
          data-vize-ui="calendar-cell"
          part="cell"
          :data-date="day.iso"
          :data-state="day.state"
          :data-outside-month="day.outsideMonth ? 'true' : undefined"
          :data-selected="day.selected ? 'true' : undefined"
          :data-in-range="day.inRange ? 'true' : undefined"
          :data-range-start="day.rangeStart ? 'true' : undefined"
          :data-range-end="day.rangeEnd ? 'true' : undefined"
          :data-preview="day.preview ? 'true' : undefined"
        >
          <button
            type="button"
            :tabindex="day.focused && !day.disabled ? 0 : -1"
            :disabled="day.disabled || day.outsideMonth"
            :aria-label="day.fullLabel"
            :aria-current="day.today ? 'date' : undefined"
            :aria-disabled="!day.disabled && (day.unavailable || day.readOnly) ? 'true' : undefined"
            data-vize-ui="calendar-day"
            part="day"
            :data-date="day.iso"
            :data-state="day.state"
            :data-weekday="day.weekday"
            :data-today="day.today ? 'true' : undefined"
            :data-focused="day.focused ? 'true' : undefined"
            :data-outside-month="day.outsideMonth ? 'true' : undefined"
            :data-selected="day.selected ? 'true' : undefined"
            :data-in-range="day.inRange ? 'true' : undefined"
            :data-range-start="day.rangeStart ? 'true' : undefined"
            :data-range-end="day.rangeEnd ? 'true' : undefined"
            :data-preview="day.preview ? 'true' : undefined"
            :data-disabled="day.disabled ? 'true' : undefined"
            :data-unavailable="day.unavailable ? 'true' : undefined"
            @click="(event) => onDayClick(day, event)"
            @keydown="(event) => onDayKeydown(day, event)"
            @pointerenter="() => onDayPointerEnter(day)"
          >
            <slot name="day" v-bind="day">{{ day.label }}</slot>
          </button>
        </td>
      </tr>
    </tbody>
  </table>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
