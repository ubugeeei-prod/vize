<script setup lang="ts">
import { computed } from "vue";

import { schedulerContext } from "./scheduler-context.ts";
import type { RowPlacement } from "./scheduler-layout.ts";
import SchedulerMonthDay from "./scheduler-month-day.vue";
import type { SchedulerDay } from "./scheduler-types.ts";

defineSlots<{
  /** Weekday column header content. */
  weekday(props: SchedulerDay): unknown;
  /** Day number content. */
  day(props: SchedulerDay): unknown;
  /** Event bar content. */
  event(props: RowPlacement<unknown>): unknown;
  /** Overflow indicator content. */
  more(props: { readonly count: number; readonly day: SchedulerDay }): unknown;
}>();

const context = schedulerContext.use();
const state = computed(() => context.slotState.value);
const headerDays = computed(() => state.value.weeks[0]?.days ?? []);
</script>

<template>
  <table
    role="grid"
    :aria-labelledby="context.headingId.value"
    :aria-readonly="state.readOnly ? 'true' : undefined"
    data-vize-ui="scheduler-month-grid"
    part="month-grid"
    :data-weeks="state.weeks.length"
  >
    <thead>
      <tr data-vize-ui="scheduler-weekdays" part="weekdays">
        <th
          v-for="day in headerDays"
          :key="day.iso"
          scope="col"
          data-vize-ui="scheduler-weekday"
          part="weekday"
        >
          <slot name="weekday" v-bind="day">{{ day.weekdayLabel }}</slot>
        </th>
      </tr>
    </thead>
    <tbody>
      <tr
        v-for="(week, row) in state.weeks"
        :key="week.days[0]?.iso ?? row"
        data-vize-ui="scheduler-week"
        part="week"
      >
        <SchedulerMonthDay v-for="(day, index) in week.days" :key="day.iso" :week :index>
          <template #day="dayState: SchedulerDay"
            ><slot name="day" v-bind="dayState">{{ dayState.dayLabel }}</slot></template
          >
          <template #event="placement: RowPlacement<unknown>"
            ><slot name="event" v-bind="placement">{{ placement.event.title }}</slot></template
          >
          <template #more="more: { readonly count: number; readonly day: SchedulerDay }"
            ><slot name="more" v-bind="more">+{{ more.count }}</slot></template
          >
        </SchedulerMonthDay>
      </tr>
    </tbody>
  </table>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
