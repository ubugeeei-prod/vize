<script setup lang="ts">
import { computed, useTemplateRef } from "vue";
import type { ComputedRef } from "vue";

import CalendarGrid from "../calendar/calendar-grid.vue";
import CalendarHeading from "../calendar/calendar-heading.vue";
import CalendarNext from "../calendar/calendar-next.vue";
import CalendarPrev from "../calendar/calendar-prev.vue";
import { useCalendarCore } from "../calendar/calendar-runtime.ts";
import type { CalendarWeekdayFormat } from "../calendar/calendar-locale.ts";
import type { DateTimeNow } from "../calendar/calendar-today.ts";
import type { CalendarDirection, CalendarSlotState } from "../calendar/calendar-types.ts";
import { formatIsoDate } from "../calendar/plain-date.ts";
import type { DateMatcher, DateRange, PlainDate, Weekday } from "../calendar/plain-date.ts";
import { useRangeCalendarSelection } from "./range-calendar-selection.ts";
import type { RangeCalendarRootExpose } from "./range-calendar-types.ts";

const props = defineProps<{
  /** Consumer-owned base id; nullish values use a deterministic fallback. @default undefined */
  readonly id?: string | null | undefined;
  /** Controlled range; `undefined` selects uncontrolled mode and `null` clears. @default undefined */
  readonly modelValue?: DateRange | null | undefined;
  /** Initial uncontrolled range. @default null */
  readonly defaultValue?: DateRange | null | undefined;
  /** Allow committed ranges to span unavailable dates. @default false */
  readonly allowNonContiguousRanges?: boolean | undefined;
  /** Controlled keyboard focus date; also decides which months are visible. @default undefined */
  readonly focusedDate?: PlainDate | null | undefined;
  /** Earliest selectable date, inclusive. @default undefined */
  readonly min?: PlainDate | null | undefined;
  /** Latest selectable date, inclusive. @default undefined */
  readonly max?: PlainDate | null | undefined;
  /** Predicate for dates that stay focusable but cannot be selected. @default undefined */
  readonly isDateUnavailable?: DateMatcher | undefined;
  /** BCP 47 locale; defaults to the nearest LocaleProvider. @default undefined */
  readonly locale?: string | undefined;
  /** Intl calendar used for display labels only, for example `japanese`. @default undefined */
  readonly calendar?: string | undefined;
  /** Intl numbering system for labels. @default undefined */
  readonly numberingSystem?: string | undefined;
  /** Text direction; defaults to the nearest LocaleProvider. @default undefined */
  readonly dir?: CalendarDirection | undefined;
  /** First day of week (`0` = Sunday); defaults to the locale preference. @default undefined */
  readonly weekStartsOn?: Weekday | undefined;
  /** Weekday column label width. @default "short" */
  readonly weekdayFormat?: CalendarWeekdayFormat | undefined;
  /** Number of consecutive months rendered. @default 1 */
  readonly numberOfMonths?: number | undefined;
  /** Move month controls by `numberOfMonths` instead of one month. @default false */
  readonly pagedNavigation?: boolean | undefined;
  /** Always render six week rows per month. @default false */
  readonly fixedWeeks?: boolean | undefined;
  /** Explicit current date; the SSR-safe way to mark today. @default undefined */
  readonly today?: PlainDate | null | undefined;
  /** Injectable clock evaluated during setup on server and client. @default undefined */
  readonly now?: DateTimeNow | undefined;
  /** IANA time zone used with `now` and the post-mount host clock. @default undefined */
  readonly timeZone?: string | undefined;
  /** Disable navigation, focus, and selection. @default false */
  readonly disabled?: boolean | undefined;
  /** Allow navigation while blocking selection. @default false */
  readonly readOnly?: boolean | undefined;
  /** Hidden input name that submits the ISO start date with forms. @default undefined */
  readonly startName?: string | undefined;
  /** Hidden input name that submits the ISO end date with forms. @default undefined */
  readonly endName?: string | undefined;
  /** Accessible name for the calendar group. @default undefined */
  readonly ariaLabel?: string | undefined;
  /** Ids that label the calendar group; defaults to the heading. @default undefined */
  readonly ariaLabelledby?: string | undefined;
  /** Ids that describe the calendar group. @default undefined */
  readonly ariaDescribedby?: string | undefined;
}>();

const emit = defineEmits<{
  /** Fired when the calendar requests a new controlled range. */
  "update:modelValue": [value: DateRange | null];
  /** Fired after a distinct committed range with the previous range and triggering event. */
  change: [value: DateRange | null, previous: DateRange | null, nativeEvent: Event | null];
  /** Fired for every completed range selection, even when the range is unchanged. */
  select: [value: DateRange, nativeEvent: Event];
  /** Fired when the first endpoint of a new range is picked, or `null` when cancelled or completed. */
  "anchor-change": [anchor: PlainDate | null];
  /** Fired when keyboard or navigation moves the focus date. */
  "update:focusedDate": [date: PlainDate];
}>();

defineSlots<{
  /** Range calendar composition. Receives months, weekdays, heading, and state; defaults to a header plus one grid per month. */
  default(props: CalendarSlotState): unknown;
}>();

const root = useTemplateRef<HTMLDivElement>("root");
const selection = useRangeCalendarSelection({
  value: () => props.modelValue,
  defaultValue: () => props.defaultValue,
  allowNonContiguousRanges: () => props.allowNonContiguousRanges === true,
  isDateUnavailable: () => props.isDateUnavailable,
  onUpdate: (value) => emit("update:modelValue", value),
  onChange: (value, previous, event) => emit("change", value, previous, event),
  onSelect: (value, event) => emit("select", value, event),
  onAnchorChange: (anchor) => emit("anchor-change", anchor),
});
const core = useCalendarCore(props, selection.adapter, root, (date) =>
  emit("update:focusedDate", date),
);
const { id, headingId, slotState, direction, state } = core;
const isoStart = computed(() =>
  selection.value.value ? formatIsoDate(selection.value.value.start) : "",
);
const isoEnd = computed(() =>
  selection.value.value ? formatIsoDate(selection.value.value.end) : "",
);

type RangeCalendarRootSetupExpose = {
  readonly [Key in keyof RangeCalendarRootExpose]: Key extends keyof typeof core.exposed
    ? (typeof core.exposed)[Key]
    : Key extends "value"
      ? ComputedRef<DateRange | null>
      : Key extends "anchor"
        ? ComputedRef<PlainDate | null>
        : RangeCalendarRootExpose[Key];
};

const exposed = {
  ...core.exposed,
  value: selection.value,
  anchor: selection.anchor,
  setValue: (value: DateRange | null) => selection.setValue(value),
  cancel: () => selection.adapter.cancel(),
} satisfies RangeCalendarRootSetupExpose;

defineExpose(exposed);
</script>

<template>
  <div
    :id="id"
    ref="root"
    role="group"
    :dir="direction"
    :aria-label="props.ariaLabel"
    :aria-labelledby="props.ariaLabelledby ?? (props.ariaLabel ? undefined : headingId)"
    :aria-describedby="props.ariaDescribedby"
    :aria-disabled="slotState.disabled ? 'true' : undefined"
    data-vize-ui="calendar"
    part="root"
    data-mode="range"
    :data-state="state"
    :data-dir="direction"
    :data-months="slotState.months.length"
    :data-start="isoStart || undefined"
    :data-end="isoEnd || undefined"
    :data-anchor="selection.anchor.value ? formatIsoDate(selection.anchor.value) : undefined"
    :data-disabled="slotState.disabled ? 'true' : undefined"
    :data-readonly="slotState.readOnly ? 'true' : undefined"
    :data-pending="slotState.pending ? 'true' : undefined"
  >
    <slot v-bind="slotState">
      <div data-vize-ui="calendar-header" part="header">
        <CalendarPrev />
        <CalendarHeading />
        <CalendarNext />
      </div>
      <CalendarGrid
        v-for="month in slotState.months"
        :key="month.index"
        :month-index="month.index"
      />
    </slot>
    <input
      v-if="props.startName"
      type="hidden"
      :name="props.startName"
      :value="isoStart"
      :disabled="slotState.disabled"
      data-vize-ui="calendar-input"
      data-boundary="start"
    />
    <input
      v-if="props.endName"
      type="hidden"
      :name="props.endName"
      :value="isoEnd"
      :disabled="slotState.disabled"
      data-vize-ui="calendar-input"
      data-boundary="end"
    />
  </div>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
