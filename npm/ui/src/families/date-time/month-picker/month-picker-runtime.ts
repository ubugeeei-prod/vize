import { computed } from "vue";
import type { ComputedRef, ShallowRef } from "vue";

import { useControllableState } from "../../foundations/controllable-state/controllable-state.ts";
import {
  deriveDeterministicId,
  useDeterministicId,
} from "../../foundations/id/deterministic-id.ts";
import { resolveLocale, useLocaleValue } from "../../i18n/locale/locale-runtime.ts";
import { createCalendarFormatters } from "../calendar/calendar-locale.ts";
import { useToday } from "../calendar/calendar-today.ts";
import type { DateTimeNow } from "../calendar/calendar-today.ts";
import { toUtcDate } from "../calendar/plain-date.ts";
import type { PlainDate, PlainYearMonth } from "../calendar/plain-date.ts";
import type {
  MonthPickerCellState,
  MonthPickerExpose,
  MonthPickerFormat,
  MonthPickerSlotState,
  PeriodPickerState,
} from "./month-picker-types.ts";
import { usePeriodGrid } from "./period-grid-runtime.ts";

/** Props read by {@link useMonthPicker}; every key is present and may be undefined. */
export interface MonthPickerRuntimeProps {
  readonly id: string | null | undefined;
  readonly modelValue: PlainYearMonth | null | undefined;
  readonly defaultValue: PlainYearMonth | null | undefined;
  readonly min: PlainYearMonth | null | undefined;
  readonly max: PlainYearMonth | null | undefined;
  readonly isMonthUnavailable: ((value: PlainYearMonth) => boolean) | undefined;
  readonly locale: string | undefined;
  readonly dir: "ltr" | "rtl" | undefined;
  readonly calendar: string | undefined;
  readonly numberingSystem: string | undefined;
  readonly monthFormat: MonthPickerFormat | undefined;
  readonly columns: number | undefined;
  readonly today: PlainDate | null | undefined;
  readonly now: DateTimeNow | undefined;
  readonly timeZone: string | undefined;
  readonly disabled: boolean | undefined;
  readonly readOnly: boolean | undefined;
}

type MonthPickerEmit = {
  (event: "update:modelValue", value: PlainYearMonth | null): void;
  (
    event: "change",
    value: PlainYearMonth | null,
    previous: PlainYearMonth | null,
    nativeEvent: Event | null,
  ): void;
  (event: "update:focusedMonth", value: PlainYearMonth): void;
};

type MonthPickerSetupExpose = {
  readonly [Key in keyof MonthPickerExpose]: MonthPickerExpose[Key] extends (
    ...args: never[]
  ) => unknown
    ? MonthPickerExpose[Key]
    : Key extends "root"
      ? Readonly<ShallowRef<HTMLDivElement | null>>
      : ComputedRef<MonthPickerExpose[Key]>;
};

/** Linear month unit (`year * 12 + month - 1`). */
export function toMonthUnit(value: PlainYearMonth): number {
  return value.year * 12 + value.month - 1;
}

/** Year-month of a linear month unit. */
export function fromMonthUnit(unit: number): PlainYearMonth {
  const year = Math.floor(unit / 12);
  return Object.freeze({ year, month: unit - year * 12 + 1 });
}

/** Copy a `{ year, month }` record; invalid input returns `null`. */
export function normalizeYearMonth(
  value: PlainYearMonth | null | undefined,
): PlainYearMonth | null {
  if (typeof value !== "object" || value === null) return null;
  const { year, month } = value;
  return Number.isInteger(year) && Number.isInteger(month) && month >= 1 && month <= 12
    ? Object.freeze({ year, month })
    : null;
}

/** Format `YYYY-MM` (HTML `month` input value). */
export function formatIsoYearMonth(value: PlainYearMonth): string {
  return `${String(value.year).padStart(4, "0")}-${String(value.month).padStart(2, "0")}`;
}

/** Parse `YYYY-MM`; anything else returns `null`. */
export function parseIsoYearMonth(value: string): PlainYearMonth | null {
  const match = /^(\d{4})-(\d{2})$/u.exec(value.trim());
  return match ? normalizeYearMonth({ year: Number(match[1]), month: Number(match[2]) }) : null;
}

function unitOrNull(value: PlainYearMonth | null | undefined): number | null {
  const normalized = normalizeYearMonth(value);
  return normalized ? toMonthUnit(normalized) : null;
}

/** State machine behind MonthPicker. Must run during setup. */
export function useMonthPicker(
  props: MonthPickerRuntimeProps,
  emit: MonthPickerEmit,
  root: Readonly<ShallowRef<HTMLDivElement | null>>,
) {
  const localeValue = useLocaleValue();
  const id = useDeterministicId({ id: () => props.id, hint: "month-picker" });
  const headingId = computed(() => deriveDeterministicId(id.value, "heading"));
  const locale = computed(() => resolveLocale(props.locale ?? localeValue.value.locale));
  const direction = computed<"ltr" | "rtl">(() =>
    props.dir === "rtl" || props.dir === "ltr" ? props.dir : localeValue.value.direction,
  );
  const formatters = computed(() =>
    createCalendarFormatters({
      locale: locale.value,
      calendar: props.calendar ?? localeValue.value.calendar,
      numberingSystem: props.numberingSystem ?? localeValue.value.numberingSystem,
    }),
  );
  const monthFormatter = computed(() => {
    const options: Intl.DateTimeFormatOptions = {
      month: props.monthFormat ?? "short",
      timeZone: "UTC",
    };
    const calendar = props.calendar ?? localeValue.value.calendar;
    const numberingSystem = props.numberingSystem ?? localeValue.value.numberingSystem;
    if (calendar) options.calendar = calendar;
    if (numberingSystem) options.numberingSystem = numberingSystem;
    try {
      return new Intl.DateTimeFormat(locale.value, options);
    } catch {
      return new Intl.DateTimeFormat("en-US", { month: "short", timeZone: "UTC" });
    }
  });
  const today = useToday({
    today: () => props.today,
    now: () => props.now,
    timeZone: () => props.timeZone,
  });
  const disabled = computed(() => props.disabled === true);
  const readOnly = computed(() => props.readOnly === true);
  const valueState = useControllableState<PlainYearMonth | null>({
    value: () =>
      props.modelValue === undefined ? undefined : normalizeYearMonth(props.modelValue),
    defaultValue: () => normalizeYearMonth(props.defaultValue),
    equals: (left, right) => unitOrNull(left) === unitOrNull(right),
    onChange: (value) => emit("update:modelValue", value),
  });
  const value = computed(() => valueState.value.value);

  function setValue(next: PlainYearMonth | null, event: Event | null = null): boolean {
    const normalized = normalizeYearMonth(next);
    const previous = value.value;
    const changed = valueState.set(normalized);
    if (changed) emit("change", normalized, previous, event);
    return changed;
  }

  const grid = usePeriodGrid({
    root,
    pageSize: 12,
    bigStep: 120,
    columns: () => props.columns ?? 3,
    min: () => unitOrNull(props.min),
    max: () => unitOrNull(props.max),
    isUnavailable: (unit) => props.isMonthUnavailable?.(fromMonthUnit(unit)) === true,
    selected: () => unitOrNull(value.value),
    current: () => (today.value ? toMonthUnit(today.value) : null),
    focusedUnit: () => undefined,
    disabled: () => disabled.value,
    readOnly: () => readOnly.value,
    direction: () => direction.value,
    onFocusChange: (unit) => emit("update:focusedMonth", fromMonthUnit(unit)),
    onSelect: (unit, event) => {
      setValue(fromMonthUnit(unit), event);
    },
  });

  const year = computed(() =>
    grid.pageStart.value === null ? null : fromMonthUnit(grid.pageStart.value).year,
  );
  const rows = computed<readonly (readonly MonthPickerCellState[])[]>(() =>
    grid.rows.value.map((row) =>
      row.map((cell): MonthPickerCellState => {
        const period = fromMonthUnit(cell.unit);
        const date = toUtcDate({ year: period.year, month: period.month, day: 1 });
        return {
          ...cell,
          year: period.year,
          month: period.month,
          label: monthFormatter.value.format(date),
          fullLabel: formatters.value.monthYear(period),
        };
      }),
    ),
  );
  const state = computed<PeriodPickerState>(() => {
    if (disabled.value) return "disabled";
    if (year.value === null) return "pending";
    if (readOnly.value) return "readonly";
    return value.value ? "selected" : "empty";
  });
  const slotState = computed<MonthPickerSlotState>(() => ({
    value: value.value,
    year: year.value,
    heading: year.value === null ? "" : formatters.value.year(year.value),
    rows: rows.value,
    canGoPrevious: grid.canPage(-1),
    canGoNext: grid.canPage(1),
    state: state.value,
  }));
  const isoValue = computed(() => (value.value ? formatIsoYearMonth(value.value) : ""));

  const exposed = {
    root,
    value,
    year,
    heading: computed(() => slotState.value.heading),
    rows,
    canGoPrevious: computed(() => slotState.value.canGoPrevious),
    canGoNext: computed(() => slotState.value.canGoNext),
    state,
    focus: grid.focus,
    setValue: (next: PlainYearMonth | null) => setValue(next),
    navigate: grid.page,
  } satisfies MonthPickerSetupExpose;

  return { id, headingId, direction, grid, slotState, isoValue, exposed, disabled, readOnly };
}
