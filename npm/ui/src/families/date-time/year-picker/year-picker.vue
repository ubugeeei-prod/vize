<script setup lang="ts">
import { useTemplateRef } from "vue";

import type { DateTimeNow } from "../calendar/calendar-today.ts";
import type { PlainDate } from "../calendar/plain-date.ts";
import { useYearPicker } from "./year-picker-runtime.ts";
import type { YearPickerCellState, YearPickerSlotState } from "./year-picker-types.ts";

const props = defineProps<{
  /** Consumer-owned base id; nullish values use a deterministic fallback. @default undefined */
  readonly id?: string | null | undefined;
  /** Controlled ISO year; `undefined` selects uncontrolled mode and `null` clears. @default undefined */
  readonly modelValue?: number | null | undefined;
  /** Initial uncontrolled year. @default null */
  readonly defaultValue?: number | null | undefined;
  /** Earliest selectable year. @default undefined */
  readonly min?: number | null | undefined;
  /** Latest selectable year. @default undefined */
  readonly max?: number | null | undefined;
  /** Predicate for years that stay focusable but cannot be selected. @default undefined */
  readonly isYearUnavailable?: ((year: number) => boolean) | undefined;
  /** Years per page; pages align to multiples of this size. Read once at setup. @default 12 */
  readonly pageSize?: number | undefined;
  /** BCP 47 locale; defaults to the nearest LocaleProvider. @default undefined */
  readonly locale?: string | undefined;
  /** Text direction; defaults to the nearest LocaleProvider. @default undefined */
  readonly dir?: "ltr" | "rtl" | undefined;
  /** Intl calendar used for display labels only. @default undefined */
  readonly calendar?: string | undefined;
  /** Intl numbering system for labels. @default undefined */
  readonly numberingSystem?: string | undefined;
  /** Years per grid row. @default 3 */
  readonly columns?: number | undefined;
  /** Explicit current date; the SSR-safe way to mark the current year. @default undefined */
  readonly today?: PlainDate | null | undefined;
  /** Injectable clock evaluated during setup on server and client. @default undefined */
  readonly now?: DateTimeNow | undefined;
  /** IANA time zone used with `now` and the post-mount host clock. @default undefined */
  readonly timeZone?: string | undefined;
  /** Disable navigation, focus, and selection. @default false */
  readonly disabled?: boolean | undefined;
  /** Allow navigation while blocking selection. @default false */
  readonly readOnly?: boolean | undefined;
  /** Hidden input name that submits the year. @default undefined */
  readonly name?: string | undefined;
  /** Accessible name of the previous-page control. @default "Previous years" */
  readonly previousLabel?: string | undefined;
  /** Accessible name of the next-page control. @default "Next years" */
  readonly nextLabel?: string | undefined;
  /** Accessible name for the picker group. @default undefined */
  readonly ariaLabel?: string | undefined;
  /** Ids that label the picker group; defaults to the page heading. @default undefined */
  readonly ariaLabelledby?: string | undefined;
  /** Ids that describe the picker group. @default undefined */
  readonly ariaDescribedby?: string | undefined;
}>();

const emit = defineEmits<{
  /** Fired when the picker requests a new controlled year. */
  "update:modelValue": [value: number | null];
  /** Fired after a distinct selection with the previous year and triggering event. */
  change: [value: number | null, previous: number | null, nativeEvent: Event | null];
  /** Fired when keyboard or navigation moves the focused year. */
  "update:focusedYear": [value: number];
}>();

defineSlots<{
  /** Page heading content. Receives picker state. */
  heading(props: YearPickerSlotState): unknown;
  /** Previous-page control content. */
  previous(props: YearPickerSlotState): unknown;
  /** Next-page control content. */
  next(props: YearPickerSlotState): unknown;
  /** Year cell content. Receives the cell state; defaults to the localized year. */
  cell(props: YearPickerCellState): unknown;
}>();

const root = useTemplateRef<HTMLDivElement>("root");
const { id, headingId, direction, grid, slotState, isoValue, exposed } = useYearPicker(
  props,
  emit,
  root,
);

function onPrevious(): void {
  grid.page(-1);
}

function onNext(): void {
  grid.page(1);
}

function onCellClick(cell: YearPickerCellState, event: MouseEvent): void {
  grid.onCellClick(cell, event);
}

function onCellKeydown(cell: YearPickerCellState, event: KeyboardEvent): void {
  grid.onCellKeydown(cell, event);
}

defineExpose(exposed);
</script>

<template>
  <div
    :id="id"
    ref="root"
    part="root"
    role="group"
    :dir="direction"
    :aria-label="props.ariaLabel"
    :aria-labelledby="props.ariaLabelledby ?? (props.ariaLabel ? undefined : headingId)"
    :aria-describedby="props.ariaDescribedby"
    :aria-disabled="slotState.state === 'disabled' ? 'true' : undefined"
    data-vize-ui="year-picker"
    :data-state="slotState.state"
    :data-first-year="slotState.firstYear ?? undefined"
    :data-value="isoValue || undefined"
  >
    <div data-vize-ui="year-picker-header" part="header">
      <button
        type="button"
        :disabled="!slotState.canGoPrevious"
        :aria-label="props.previousLabel ?? 'Previous years'"
        :aria-controls="id"
        data-vize-ui="year-picker-previous"
        part="previous"
        @click="onPrevious"
      >
        <slot name="previous" v-bind="slotState">‹</slot>
      </button>
      <h2
        :id="headingId"
        aria-live="polite"
        aria-atomic="true"
        data-vize-ui="year-picker-heading"
        part="heading"
      >
        <slot name="heading" v-bind="slotState">{{ slotState.heading }}</slot>
      </h2>
      <button
        type="button"
        :disabled="!slotState.canGoNext"
        :aria-label="props.nextLabel ?? 'Next years'"
        :aria-controls="id"
        data-vize-ui="year-picker-next"
        part="next"
        @click="onNext"
      >
        <slot name="next" v-bind="slotState">›</slot>
      </button>
    </div>
    <table
      role="grid"
      :aria-labelledby="headingId"
      :aria-readonly="slotState.state === 'readonly' ? 'true' : undefined"
      data-vize-ui="year-picker-grid"
      part="grid"
    >
      <tbody>
        <tr
          v-for="row in slotState.rows"
          :key="row[0]?.unit"
          data-vize-ui="year-picker-row"
          part="row"
        >
          <td
            v-for="cell in row"
            :key="cell.unit"
            :aria-selected="cell.selected ? 'true' : 'false'"
            :aria-disabled="cell.disabled || cell.unavailable ? 'true' : undefined"
            data-vize-ui="year-picker-cell"
            part="cell"
          >
            <button
              type="button"
              :tabindex="cell.focused ? 0 : -1"
              :disabled="cell.disabled"
              :aria-label="cell.label"
              :aria-current="cell.current ? 'date' : undefined"
              :aria-disabled="cell.unavailable ? 'true' : undefined"
              data-vize-ui="year-picker-year"
              part="year"
              :data-unit="cell.unit"
              :data-year="cell.year"
              :data-selected="cell.selected ? 'true' : undefined"
              :data-current="cell.current ? 'true' : undefined"
              :data-focused="cell.focused ? 'true' : undefined"
              :data-disabled="cell.disabled ? 'true' : undefined"
              :data-unavailable="cell.unavailable ? 'true' : undefined"
              @click="(event) => onCellClick(cell, event)"
              @keydown="(event) => onCellKeydown(cell, event)"
            >
              <slot name="cell" v-bind="cell">{{ cell.label }}</slot>
            </button>
          </td>
        </tr>
      </tbody>
    </table>
    <input
      v-if="props.name"
      type="hidden"
      :name="props.name"
      :value="isoValue"
      :disabled="slotState.state === 'disabled'"
      data-vize-ui="year-picker-input"
    />
  </div>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
