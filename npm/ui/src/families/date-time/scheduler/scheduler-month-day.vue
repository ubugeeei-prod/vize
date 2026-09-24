<script setup lang="ts">
import { computed, onScopeDispose, useTemplateRef } from "vue";

import { schedulerContext } from "./scheduler-context.ts";
import SchedulerEvent from "./scheduler-event.vue";
import type { RowPlacement } from "./scheduler-layout.ts";
import type { SchedulerDay, SchedulerWeekState } from "./scheduler-types.ts";

const { week, index } = defineProps<{
  /** Week row that owns this day. @default undefined (required) */
  readonly week: SchedulerWeekState<unknown>;
  /** Day index inside the week. @default undefined (required) */
  readonly index: number;
}>();

defineSlots<{
  /** Day number content. */
  day(props: SchedulerDay): unknown;
  /** Event bar content for events starting in this cell. */
  event(props: RowPlacement<unknown>): unknown;
  /** Overflow indicator content when more events exist than `maxLanes`. */
  more(props: { readonly count: number; readonly day: SchedulerDay }): unknown;
}>();

const context = schedulerContext.use();
const element = useTemplateRef<HTMLTableCellElement>("element");
const day = computed<SchedulerDay | null>(() => week.days[index] ?? null);
const target = context.registerDayTarget(
  () => day.value?.date ?? { year: 1970, month: 1, day: 1 },
  "day",
  element,
);
const placements = computed(() =>
  week.placements.filter(
    (placement) => placement.lane < context.maxLanes.value && placement.startIndex === index,
  ),
);
const overflow = computed(() => week.overflow[index] ?? 0);
const focused = computed(() => day.value !== null && day.value.iso === context.focusedIso.value);

onScopeDispose(() => target.dispose());

function onClick(event: MouseEvent): void {
  if (day.value) context.onDayActivate(day.value.date, event);
}

function onKeydown(event: KeyboardEvent): void {
  if (day.value) context.onDayKeydown(day.value.date, event);
}
</script>

<template>
  <td
    ref="element"
    :aria-current="day?.today ? 'date' : undefined"
    data-vize-ui="scheduler-month-cell"
    part="month-cell"
    :data-date="day?.iso"
    :data-outside-month="day?.outsideMonth ? 'true' : undefined"
    :data-today="day?.today ? 'true' : undefined"
    :data-drop-target="target.isOver.value ? 'true' : undefined"
  >
    <button
      v-if="day"
      type="button"
      :tabindex="focused ? 0 : -1"
      :disabled="context.slotState.value.disabled"
      :aria-label="day.fullLabel"
      data-vize-ui="scheduler-day"
      part="day"
      :data-date="day.iso"
      :data-focused="focused ? 'true' : undefined"
      @click="onClick"
      @keydown="onKeydown"
    >
      <slot name="day" v-bind="day">{{ day.dayLabel }}</slot>
    </button>
    <SchedulerEvent
      v-for="placement in placements"
      :key="placement.event.id"
      :event="placement.event"
      :row="placement"
    >
      <slot name="event" v-bind="placement">{{ placement.event.title }}</slot>
    </SchedulerEvent>
    <span
      v-if="overflow > 0 && day"
      data-vize-ui="scheduler-more"
      part="more"
      :data-count="overflow"
    >
      <slot name="more" :count="overflow" :day="day">+{{ overflow }}</slot>
    </span>
  </td>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
