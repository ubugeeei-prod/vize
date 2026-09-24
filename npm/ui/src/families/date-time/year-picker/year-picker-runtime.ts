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
import type { PlainDate } from "../calendar/plain-date.ts";
import type { PeriodPickerState } from "../month-picker/month-picker-types.ts";
import { usePeriodGrid } from "../month-picker/period-grid-runtime.ts";
import type {
  YearPickerCellState,
  YearPickerExpose,
  YearPickerSlotState,
} from "./year-picker-types.ts";

/** Props read by {@link useYearPicker}; every key is present and may be undefined. */
export interface YearPickerRuntimeProps {
  readonly id: string | null | undefined;
  readonly modelValue: number | null | undefined;
  readonly defaultValue: number | null | undefined;
  readonly min: number | null | undefined;
  readonly max: number | null | undefined;
  readonly isYearUnavailable: ((year: number) => boolean) | undefined;
  readonly pageSize: number | undefined;
  readonly locale: string | undefined;
  readonly dir: "ltr" | "rtl" | undefined;
  readonly calendar: string | undefined;
  readonly numberingSystem: string | undefined;
  readonly columns: number | undefined;
  readonly today: PlainDate | null | undefined;
  readonly now: DateTimeNow | undefined;
  readonly timeZone: string | undefined;
  readonly disabled: boolean | undefined;
  readonly readOnly: boolean | undefined;
}

type YearPickerEmit = {
  (event: "update:modelValue", value: number | null): void;
  (event: "change", value: number | null, previous: number | null, nativeEvent: Event | null): void;
  (event: "update:focusedYear", value: number): void;
};

type YearPickerSetupExpose = {
  readonly [Key in keyof YearPickerExpose]: YearPickerExpose[Key] extends (
    ...args: never[]
  ) => unknown
    ? YearPickerExpose[Key]
    : Key extends "root"
      ? Readonly<ShallowRef<HTMLDivElement | null>>
      : ComputedRef<YearPickerExpose[Key]>;
};

function normalizeYear(value: number | null | undefined): number | null {
  return typeof value === "number" && Number.isInteger(value) ? value : null;
}

/** State machine behind YearPicker. Must run during setup. */
export function useYearPicker(
  props: YearPickerRuntimeProps,
  emit: YearPickerEmit,
  root: Readonly<ShallowRef<HTMLDivElement | null>>,
) {
  const localeValue = useLocaleValue();
  const id = useDeterministicId({ id: () => props.id, hint: "year-picker" });
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
  const today = useToday({
    today: () => props.today,
    now: () => props.now,
    timeZone: () => props.timeZone,
  });
  const disabled = computed(() => props.disabled === true);
  const readOnly = computed(() => props.readOnly === true);
  const valueState = useControllableState<number | null>({
    value: () => (props.modelValue === undefined ? undefined : normalizeYear(props.modelValue)),
    defaultValue: () => normalizeYear(props.defaultValue),
    onChange: (value) => emit("update:modelValue", value),
  });
  const value = computed(() => valueState.value.value);

  function setValue(next: number | null, event: Event | null = null): boolean {
    const normalized = normalizeYear(next);
    const previous = value.value;
    const changed = valueState.set(normalized);
    if (changed) emit("change", normalized, previous, event);
    return changed;
  }

  const pageSize = Math.min(100, Math.max(1, Math.trunc(props.pageSize ?? 12) || 12));
  const grid = usePeriodGrid({
    root,
    pageSize,
    bigStep: pageSize * 10,
    columns: () => props.columns ?? 3,
    min: () => normalizeYear(props.min),
    max: () => normalizeYear(props.max),
    isUnavailable: (unit) => props.isYearUnavailable?.(unit) === true,
    selected: () => value.value,
    current: () => today.value?.year ?? null,
    focusedUnit: () => undefined,
    disabled: () => disabled.value,
    readOnly: () => readOnly.value,
    direction: () => direction.value,
    onFocusChange: (unit) => emit("update:focusedYear", unit),
    onSelect: (unit, event) => {
      setValue(unit, event);
    },
  });

  const firstYear = computed(() => grid.pageStart.value);
  const lastYear = computed(() =>
    grid.pageStart.value === null ? null : grid.pageStart.value + pageSize - 1,
  );
  const rows = computed<readonly (readonly YearPickerCellState[])[]>(() =>
    grid.rows.value.map((row) =>
      row.map((cell): YearPickerCellState => ({
        ...cell,
        year: cell.unit,
        label: formatters.value.year(cell.unit),
      })),
    ),
  );
  const state = computed<PeriodPickerState>(() => {
    if (disabled.value) return "disabled";
    if (firstYear.value === null) return "pending";
    if (readOnly.value) return "readonly";
    return value.value === null ? "empty" : "selected";
  });
  const slotState = computed<YearPickerSlotState>(() => ({
    value: value.value,
    firstYear: firstYear.value,
    lastYear: lastYear.value,
    heading:
      firstYear.value === null || lastYear.value === null
        ? ""
        : `${formatters.value.year(firstYear.value)} – ${formatters.value.year(lastYear.value)}`,
    rows: rows.value,
    canGoPrevious: grid.canPage(-1),
    canGoNext: grid.canPage(1),
    state: state.value,
  }));
  const isoValue = computed(() => (value.value === null ? "" : String(value.value)));

  const exposed = {
    root,
    value,
    firstYear,
    lastYear,
    heading: computed(() => slotState.value.heading),
    rows,
    canGoPrevious: computed(() => slotState.value.canGoPrevious),
    canGoNext: computed(() => slotState.value.canGoNext),
    state,
    focus: grid.focus,
    setValue: (next: number | null) => setValue(next),
    navigate: grid.page,
  } satisfies YearPickerSetupExpose;

  return { id, headingId, direction, grid, slotState, isoValue, exposed, disabled, readOnly };
}
