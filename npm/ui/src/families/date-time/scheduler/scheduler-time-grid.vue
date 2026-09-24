<script setup lang="ts">
import { computed } from "vue";

import SchedulerDayColumn from "./scheduler-day-column.vue";
import { schedulerContext } from "./scheduler-context.ts";
import SchedulerEvent from "./scheduler-event.vue";
import type { RowPlacement, TimeGridPlacement } from "./scheduler-layout.ts";
import type { SchedulerDay, SchedulerSlotRow } from "./scheduler-types.ts";

const { timeHeader = "Time", allDayLabel = "All day" } = defineProps<{
  /** Corner column header announced for the time labels. @default "Time" */
  readonly timeHeader?: string;
  /** Accessible name of the all-day lane group. @default "All day" */
  readonly allDayLabel?: string;
}>();

defineSlots<{
  /** Day column header content. */
  dayHeader(props: SchedulerDay): unknown;
  /** Time label content for each slot row. */
  time(props: SchedulerSlotRow): unknown;
  /** Timed event content. */
  event(props: TimeGridPlacement<unknown>): unknown;
  /** All-day event content. */
  allDayEvent(props: RowPlacement<unknown>): unknown;
}>();

const context = schedulerContext.use();
const state = computed(() => context.slotState.value);
const gridStyle = computed(() => ({
  "--vize-scheduler-days": String(state.value.days.length),
  "--vize-scheduler-slots": String(state.value.slots.length),
}));

function isFocused(day: SchedulerDay, slot: SchedulerSlotRow): boolean {
  return day.iso === context.focusedIso.value && slot.minute === context.focusedMinute.value;
}

function onSlotClick(day: SchedulerDay, slot: SchedulerSlotRow, event: MouseEvent): void {
  context.onSlotActivate(day.date, slot.minute, event);
}

function onSlotKeydown(day: SchedulerDay, slot: SchedulerSlotRow, event: KeyboardEvent): void {
  context.onSlotKeydown(day.date, slot.minute, event);
}
</script>

<template>
  <div
    data-vize-ui="scheduler-time-grid"
    part="time-grid"
    :data-view="state.view"
    :data-pending="state.pending ? 'true' : undefined"
    :style="gridStyle"
  >
    <table
      role="grid"
      :aria-labelledby="context.headingId.value"
      :aria-readonly="state.readOnly ? 'true' : undefined"
      data-vize-ui="scheduler-slot-grid"
      part="slot-grid"
    >
      <thead>
        <tr data-vize-ui="scheduler-day-headers" part="day-headers">
          <th scope="col" data-vize-ui="scheduler-corner" part="corner">{{ timeHeader }}</th>
          <th
            v-for="day in state.days"
            :key="day.iso"
            scope="col"
            :abbr="day.fullLabel"
            :aria-current="day.today ? 'date' : undefined"
            data-vize-ui="scheduler-day-header"
            part="day-header"
            :data-date="day.iso"
            :data-today="day.today ? 'true' : undefined"
          >
            <slot name="dayHeader" v-bind="day">{{ day.label }}</slot>
          </th>
        </tr>
      </thead>
      <tbody>
        <tr
          v-for="slot in state.slots"
          :key="slot.minute"
          data-vize-ui="scheduler-slot-row"
          part="slot-row"
          :data-minute="slot.minute"
        >
          <th scope="row" data-vize-ui="scheduler-time" part="time">
            <slot name="time" v-bind="slot">{{ slot.label }}</slot>
          </th>
          <td
            v-for="day in state.days"
            :key="day.iso"
            data-vize-ui="scheduler-slot-cell"
            part="slot-cell"
          >
            <button
              type="button"
              :tabindex="isFocused(day, slot) ? 0 : -1"
              :disabled="state.disabled"
              :aria-label="`${day.fullLabel} ${slot.label}`"
              data-vize-ui="scheduler-slot"
              part="slot"
              :data-date="day.iso"
              :data-minute="slot.minute"
              :data-today="day.today ? 'true' : undefined"
              :data-focused="isFocused(day, slot) ? 'true' : undefined"
              @click="(event) => onSlotClick(day, slot, event)"
              @keydown="(event) => onSlotKeydown(day, slot, event)"
            ></button>
          </td>
        </tr>
      </tbody>
    </table>
    <div
      v-if="state.allDay.length > 0"
      role="group"
      :aria-label="allDayLabel"
      data-vize-ui="scheduler-all-day"
      part="all-day"
    >
      <SchedulerEvent v-for="row in state.allDay" :key="row.event.id" :event="row.event" :row>
        <slot name="allDayEvent" v-bind="row">{{ row.event.title }}</slot>
      </SchedulerEvent>
    </div>
    <div data-vize-ui="scheduler-event-layer" part="event-layer">
      <SchedulerDayColumn v-for="column in state.columns" :key="column.day.iso" :column>
        <template #event="placement: TimeGridPlacement<unknown>">
          <slot name="event" v-bind="placement">{{ placement.event.title }}</slot>
        </template>
      </SchedulerDayColumn>
    </div>
  </div>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
