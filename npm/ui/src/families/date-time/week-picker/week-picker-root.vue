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
import { addDays, formatIsoDate, formatIsoWeek, isoWeekOf } from "../calendar/plain-date.ts";
import type {
  DateMatcher,
  DateRange,
  IsoWeek,
  PlainDate,
  Weekday,
} from "../calendar/plain-date.ts";
import { useWeekSelection } from "./week-picker-selection.ts";
import type { WeekPickerRootExpose } from "./week-picker-types.ts";

const props = defineProps<{
  /** Consumer-owned base id; nullish values use a deterministic fallback. @default undefined */
  readonly id?: string | null | undefined;
  /** Controlled week as a start–end range; `undefined` selects uncontrolled mode and `null` clears. @default undefined */
  readonly modelValue?: DateRange | null | undefined;
  /** Initial uncontrolled week. @default null */
  readonly defaultValue?: DateRange | null | undefined;
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
  /** Hidden input name that submits the ISO 8601 week (`YYYY-Www`) of the selected week. @default undefined */
  readonly name?: string | undefined;
  /** Omit ISO week numbers from the default grid composition. @default false */
  readonly hideWeekNumbers?: boolean | undefined;
  /** Accessible name for the calendar group. @default undefined */
  readonly ariaLabel?: string | undefined;
  /** Ids that label the calendar group; defaults to the heading. @default undefined */
  readonly ariaLabelledby?: string | undefined;
  /** Ids that describe the calendar group. @default undefined */
  readonly ariaDescribedby?: string | undefined;
}>();

const emit = defineEmits<{
  /** Fired when the picker requests a new controlled week. */
  "update:modelValue": [value: DateRange | null];
  /** Fired after a distinct committed week with the previous week and triggering event. */
  change: [value: DateRange | null, previous: DateRange | null, nativeEvent: Event | null];
  /** Fired for every week activation, even when the week is unchanged. */
  select: [value: DateRange, nativeEvent: Event];
  /** Fired when keyboard or navigation moves the focus date. */
  "update:focusedDate": [date: PlainDate];
}>();

defineSlots<{
  /** Week picker composition. Receives months, weekdays, heading, and state; defaults to a header plus one grid per month. */
  default(props: CalendarSlotState): unknown;
}>();

const root = useTemplateRef<HTMLDivElement>("root");
let resolveWeekStart = (): Weekday => 0;
const selection = useWeekSelection({
  value: () => props.modelValue,
  defaultValue: () => props.defaultValue,
  weekStartsOn: () => resolveWeekStart(),
  onUpdate: (value) => emit("update:modelValue", value),
  onChange: (value, previous, event) => emit("change", value, previous, event),
  onSelect: (value, event) => emit("select", value, event),
});
const core = useCalendarCore(props, selection.adapter, root, (date) =>
  emit("update:focusedDate", date),
);
resolveWeekStart = () => core.exposed.weekStartsOn.value;
const { id, headingId, slotState, direction, state } = core;
const isoStart = computed(() =>
  selection.value.value ? formatIsoDate(selection.value.value.start) : "",
);
const isoWeek = computed(() =>
  selection.value.value ? isoWeekOf(addDays(selection.value.value.start, 3)) : null,
);
const isoWeekText = computed(() => (isoWeek.value ? formatIsoWeek(isoWeek.value) : ""));

type WeekPickerRootSetupExpose = {
  readonly [Key in keyof WeekPickerRootExpose]: Key extends keyof typeof core.exposed
    ? (typeof core.exposed)[Key]
    : Key extends "value"
      ? ComputedRef<DateRange | null>
      : Key extends "isoWeek"
        ? ComputedRef<IsoWeek | null>
        : WeekPickerRootExpose[Key];
};

const exposed = {
  ...core.exposed,
  value: selection.value,
  isoWeek,
  setValue: (value: DateRange | null) => selection.setValue(value),
} satisfies WeekPickerRootSetupExpose;

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
    data-vize-ui="week-picker"
    part="root"
    data-mode="week"
    :data-state="state"
    :data-dir="direction"
    :data-months="slotState.months.length"
    :data-start="isoStart || undefined"
    :data-week="isoWeekText || undefined"
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
        :show-week-numbers="!props.hideWeekNumbers"
      />
    </slot>
    <input
      v-if="props.name"
      type="hidden"
      :name="props.name"
      :value="isoWeekText"
      :disabled="slotState.disabled"
      data-vize-ui="week-picker-input"
    />
  </div>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
