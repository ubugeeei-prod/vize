import { computed, shallowRef } from "vue";
import type { ComputedRef } from "vue";

import { useControllableState } from "../../foundations/controllable-state/controllable-state.ts";
import type {
  CalendarDaySelection,
  CalendarSelectionAdapter,
} from "../calendar/calendar-runtime.ts";
import {
  createDateRange,
  endOfWeek,
  isDateInRange,
  isSameDay,
  isSameRange,
  normalizeDateRange,
  startOfWeek,
} from "../calendar/plain-date.ts";
import type { DateRange, PlainDate, Weekday } from "../calendar/plain-date.ts";

/** Week containing `date` for a locale week start. */
export function weekOf(date: PlainDate, weekStartsOn: Weekday): DateRange {
  return createDateRange(startOfWeek(date, weekStartsOn), endOfWeek(date, weekStartsOn));
}

/** Options for {@link useWeekSelection}. */
export interface WeekSelectionOptions {
  readonly value: () => DateRange | null | undefined;
  readonly defaultValue: () => DateRange | null | undefined;
  /** Week start, resolved lazily from the calendar core. */
  readonly weekStartsOn: () => Weekday;
  readonly onUpdate: (value: DateRange | null) => void;
  readonly onChange: (
    value: DateRange | null,
    previous: DateRange | null,
    event: Event | null,
  ) => void;
  readonly onSelect?: (value: DateRange, event: Event) => void;
}

/** Week selection state plus its calendar adapter. */
export interface WeekSelection {
  readonly value: ComputedRef<DateRange | null>;
  readonly adapter: CalendarSelectionAdapter;
  readonly setValue: (value: DateRange | null, event?: Event | null) => boolean;
}

/**
 * Whole-week selection: activating any day selects the week that contains it,
 * and pointer or keyboard focus previews the week under it.
 */
export function useWeekSelection(options: WeekSelectionOptions): WeekSelection {
  const state = useControllableState<DateRange | null>({
    value: () => {
      const value = options.value();
      return value === undefined ? undefined : normalizeDateRange(value);
    },
    defaultValue: () => normalizeDateRange(options.defaultValue()),
    equals: isSameRange,
    onChange: (value) => options.onUpdate(value),
  });
  const value = computed(() => state.value.value);
  const hovered = shallowRef<PlainDate | null>(null);

  function setValue(next: DateRange | null, event: Event | null = null): boolean {
    const normalized = normalizeDateRange(next);
    const previous = value.value;
    const changed = state.set(normalized);
    if (changed) options.onChange(normalized, previous, event);
    return changed;
  }

  function daySelection(date: PlainDate): CalendarDaySelection {
    const selected = value.value;
    const preview = hovered.value ? weekOf(hovered.value, options.weekStartsOn()) : null;
    const inSelected = isDateInRange(date, selected);
    const inPreview = !inSelected && isDateInRange(date, preview);
    const range = inSelected ? selected : inPreview ? preview : null;
    return {
      selected: inSelected,
      rangeStart: range !== null && isSameDay(date, range.start),
      rangeEnd: range !== null && isSameDay(date, range.end),
      inRange: inSelected || inPreview,
      preview: inPreview,
    };
  }

  return {
    value,
    setValue,
    adapter: {
      mode: "week",
      anchorDate: () => value.value?.start ?? null,
      hasSelection: () => value.value !== null,
      daySelection,
      select: (date, event) => {
        const week = weekOf(date, options.weekStartsOn());
        setValue(week, event);
        options.onSelect?.(week, event);
      },
      hover: (date) => {
        hovered.value = date;
      },
      cancel: () => false,
    },
  };
}
