<script setup lang="ts">
import { computed, useTemplateRef } from "vue";
import type { ComputedRef } from "vue";

import CalendarGrid from "./calendar-grid.vue";
import CalendarHeading from "./calendar-heading.vue";
import CalendarNext from "./calendar-next.vue";
import CalendarPrev from "./calendar-prev.vue";
import { useCalendarCore } from "./calendar-runtime.ts";
import { useMultipleCalendarSelection } from "./calendar-selection.ts";
import type { CalendarWeekdayFormat } from "./calendar-locale.ts";
import type { DateTimeNow } from "./calendar-today.ts";
import type {
  CalendarDirection,
  CalendarMultipleRootExpose,
  CalendarSlotState,
} from "./calendar-types.ts";
import { formatIsoDate } from "./plain-date.ts";
import type { DateMatcher, PlainDate, Weekday } from "./plain-date.ts";

const props = defineProps<{
  /** Consumer-owned base id; nullish values use a deterministic fallback. @default undefined */
  readonly id?: string | null | undefined;
  /** Controlled selected dates; `undefined` selects uncontrolled mode. @default undefined */
  readonly modelValue?: readonly PlainDate[] | undefined;
  /** Initial uncontrolled selected dates. @default [] */
  readonly defaultValue?: readonly PlainDate[] | undefined;
  /** Largest number of selected dates; activating another unselected date is ignored. @default undefined */
  readonly maxSelections?: number | undefined;
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
  /** Hidden input name; one input per selected ISO date is submitted. @default undefined */
  readonly name?: string | undefined;
  /** Accessible name for the calendar group. @default undefined */
  readonly ariaLabel?: string | undefined;
  /** Ids that label the calendar group; defaults to the heading. @default undefined */
  readonly ariaLabelledby?: string | undefined;
  /** Ids that describe the calendar group. @default undefined */
  readonly ariaDescribedby?: string | undefined;
}>();

const emit = defineEmits<{
  /** Fired when the calendar requests new controlled dates (sorted, unique). */
  "update:modelValue": [value: readonly PlainDate[]];
  /** Fired after a distinct selection with the previous dates and triggering event. */
  change: [value: readonly PlainDate[], previous: readonly PlainDate[], nativeEvent: Event | null];
  /** Fired after a user activation toggles a date, with its new selected state. */
  toggle: [date: PlainDate, selected: boolean, nativeEvent: Event];
  /** Fired when keyboard or navigation moves the focus date. */
  "update:focusedDate": [date: PlainDate];
}>();

defineSlots<{
  /** Calendar composition. Receives months, weekdays, heading, and state; defaults to a header plus one grid per month. */
  default(props: CalendarSlotState): unknown;
}>();

const root = useTemplateRef<HTMLDivElement>("root");
const selection = useMultipleCalendarSelection({
  value: () => props.modelValue,
  defaultValue: () => props.defaultValue,
  maxSelections: () => props.maxSelections,
  onUpdate: (value) => emit("update:modelValue", value),
  onChange: (value, previous, event) => emit("change", value, previous, event),
  onToggle: (date, selected, event) => emit("toggle", date, selected, event),
});
const core = useCalendarCore(props, selection.adapter, root, (date) =>
  emit("update:focusedDate", date),
);
const { id, headingId, slotState, direction, state } = core;
const isoValues = computed(() => selection.value.value.map((date) => formatIsoDate(date)));

type CalendarMultipleRootSetupExpose = {
  readonly [Key in keyof CalendarMultipleRootExpose]: Key extends keyof typeof core.exposed
    ? (typeof core.exposed)[Key]
    : Key extends "value"
      ? ComputedRef<readonly PlainDate[]>
      : CalendarMultipleRootExpose[Key];
};

const exposed = {
  ...core.exposed,
  value: selection.value,
  setValue: (value: readonly PlainDate[]) => selection.setValue(value),
  toggle: (date: PlainDate) => selection.toggle(date),
} satisfies CalendarMultipleRootSetupExpose;

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
    data-mode="multiple"
    :data-state="state"
    :data-dir="direction"
    :data-months="slotState.months.length"
    :data-count="isoValues.length"
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
      v-for="iso in props.name ? isoValues : []"
      :key="iso"
      type="hidden"
      :name="props.name"
      :value="iso"
      :disabled="slotState.disabled"
      data-vize-ui="calendar-input"
    />
  </div>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
