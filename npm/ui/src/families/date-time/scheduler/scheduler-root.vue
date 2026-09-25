<script setup lang="ts" generic="Data">
import { useTemplateRef } from "vue";

import type { DateTimeNow } from "../calendar/calendar-today.ts";
import type { PlainDate, Weekday } from "../calendar/plain-date.ts";
import SchedulerHeading from "./scheduler-heading.vue";
import type { SchedulerEvent, SchedulerView } from "./scheduler-layout.ts";
import SchedulerMonthGrid from "./scheduler-month-grid.vue";
import SchedulerNav from "./scheduler-nav.vue";
import { useScheduler } from "./scheduler-runtime.ts";
import SchedulerTimeGrid from "./scheduler-time-grid.vue";
import type {
  SchedulerEventChange,
  SchedulerSlotRange,
  SchedulerSlotState,
} from "./scheduler-types.ts";

const props = defineProps<{
  /** Consumer-owned base id; nullish values use a deterministic fallback. @default undefined */
  readonly id?: string | null | undefined;
  /** Events to lay out; `data` keeps the consumer payload type. @default [] */
  readonly events?: readonly SchedulerEvent<Data>[] | undefined;
  /** Controlled view; `undefined` selects uncontrolled mode. @default undefined */
  readonly view?: SchedulerView | undefined;
  /** Initial uncontrolled view. @default "week" */
  readonly defaultView?: SchedulerView | undefined;
  /** Controlled anchor date deciding the visible period. @default undefined */
  readonly date?: PlainDate | null | undefined;
  /** Initial uncontrolled anchor date. @default undefined */
  readonly defaultDate?: PlainDate | null | undefined;
  /** Explicit current date; the SSR-safe way to anchor and mark today. @default undefined */
  readonly today?: PlainDate | null | undefined;
  /** Injectable clock evaluated during setup on server and client. @default undefined */
  readonly now?: DateTimeNow | undefined;
  /** IANA time zone used with `now` and the post-mount host clock. @default undefined */
  readonly timeZone?: string | undefined;
  /** BCP 47 locale; defaults to the nearest LocaleProvider. @default undefined */
  readonly locale?: string | undefined;
  /** Text direction; defaults to the nearest LocaleProvider. @default undefined */
  readonly dir?: "ltr" | "rtl" | undefined;
  /** First day of week; defaults to the locale preference. @default undefined */
  readonly weekStartsOn?: Weekday | undefined;
  /** First visible hour of the time grid. @default 0 */
  readonly dayStartHour?: number | undefined;
  /** Hour where the time grid ends, exclusive. @default 24 */
  readonly dayEndHour?: number | undefined;
  /** Minutes per time-grid slot row. @default 30 */
  readonly slotMinutes?: number | undefined;
  /** Minutes that drag and keyboard moves snap to. @default slotMinutes */
  readonly snapMinutes?: number | undefined;
  /** Minimum rendered event length in minutes. @default 15 */
  readonly minimumEventMinutes?: number | undefined;
  /** Month-view lanes rendered per week before counting overflow. @default 3 */
  readonly maxLanes?: number | undefined;
  /** Disable navigation, activation, and editing. @default false */
  readonly disabled?: boolean | undefined;
  /** Keep navigation and activation but block move and resize requests. @default false */
  readonly readOnly?: boolean | undefined;
  /** Accessible name for the scheduler region. @default undefined */
  readonly ariaLabel?: string | undefined;
}>();

const emit = defineEmits<{
  /** Fired when the scheduler requests a new controlled view. */
  "update:view": [view: SchedulerView];
  /** Fired when navigation requests a new anchor date. */
  "update:date": [date: PlainDate];
  /** Fired when an event is clicked or activated with Enter/Space. */
  "event-activate": [event: SchedulerEvent<Data>, nativeEvent: Event];
  /** Fired when a time-grid slot is clicked or activated. */
  "slot-activate": [range: SchedulerSlotRange, nativeEvent: Event];
  /** Fired when a month-view day is clicked or activated. */
  "day-activate": [date: PlainDate, nativeEvent: Event];
  /** Fired when a drag or Alt+Arrow requests a new start (duration preserved). */
  "event-move": [change: SchedulerEventChange<Data>, nativeEvent: Event | null];
  /** Fired when a resize drag or Alt+Shift+Arrow requests a new end. */
  "event-resize": [change: SchedulerEventChange<Data>, nativeEvent: Event | null];
}>();

defineSlots<{
  /** Scheduler composition. Receives typed layout state; defaults to a header plus the view's grid. */
  default?(props: SchedulerSlotState<Data>): unknown;
}>();

const root = useTemplateRef<HTMLDivElement>("root");
const { id, slotState, direction, exposed } = useScheduler<Data>(
  props,
  {
    view: (view) => emit("update:view", view),
    date: (date) => emit("update:date", date),
    eventActivate: (event, nativeEvent) => emit("event-activate", event, nativeEvent),
    slotActivate: (range, nativeEvent) => emit("slot-activate", range, nativeEvent),
    dayActivate: (date, nativeEvent) => emit("day-activate", date, nativeEvent),
    eventMove: (change, nativeEvent) => emit("event-move", change, nativeEvent),
    eventResize: (change, nativeEvent) => emit("event-resize", change, nativeEvent),
  },
  root,
);

defineExpose(exposed);
</script>

<template>
  <div
    :id="id"
    ref="root"
    part="root"
    role="region"
    :dir="direction"
    :aria-label="props.ariaLabel"
    aria-roledescription="scheduler"
    data-vize-ui="scheduler"
    :data-state="slotState.state"
    :data-view="slotState.view"
    :data-dir="direction"
    :data-disabled="slotState.disabled ? 'true' : undefined"
    :data-readonly="slotState.readOnly ? 'true' : undefined"
    :data-pending="slotState.pending ? 'true' : undefined"
  >
    <slot v-bind="slotState">
      <div data-vize-ui="scheduler-header" part="header">
        <SchedulerNav action="previous" />
        <SchedulerNav action="today" />
        <SchedulerNav action="next" />
        <SchedulerHeading />
      </div>
      <SchedulerMonthGrid v-if="slotState.view === 'month'" />
      <SchedulerTimeGrid v-else />
    </slot>
  </div>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
