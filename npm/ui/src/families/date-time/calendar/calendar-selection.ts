import { computed } from "vue";
import type { ComputedRef } from "vue";

import { useControllableState } from "../../foundations/controllable-state/controllable-state.ts";
import type { CalendarDaySelection, CalendarSelectionAdapter } from "./calendar-runtime.ts";
import { isSameDay, normalizePlainDate } from "./plain-date.ts";
import type { PlainDate } from "./plain-date.ts";

/** Options for {@link useSingleCalendarSelection}. */
export interface SingleCalendarSelectionOptions {
  /** Controlled value; `undefined` selects uncontrolled mode. */
  readonly value: () => PlainDate | null | undefined;
  /** Initial uncontrolled value. */
  readonly defaultValue: () => PlainDate | null | undefined;
  /** Called for every distinct requested value. */
  readonly onUpdate: (value: PlainDate | null) => void;
  /** Called after a distinct user or imperative change. */
  readonly onChange: (
    value: PlainDate | null,
    previous: PlainDate | null,
    event: Event | null,
  ) => void;
  /** Called for every user activation of a selectable date, even when unchanged. */
  readonly onSelect?: (value: PlainDate, event: Event) => void;
}

/** Single-date selection state plus its calendar adapter. */
export interface SingleCalendarSelection {
  readonly value: ComputedRef<PlainDate | null>;
  readonly adapter: CalendarSelectionAdapter;
  readonly setValue: (value: PlainDate | null, event?: Event | null) => boolean;
  readonly reset: () => boolean;
}

const unselected: CalendarDaySelection = Object.freeze({
  selected: false,
  rangeStart: false,
  rangeEnd: false,
  inRange: false,
  preview: false,
});
const selectedDay: CalendarDaySelection = Object.freeze({
  selected: true,
  rangeStart: true,
  rangeEnd: true,
  inRange: true,
  preview: false,
});

/** Controlled/uncontrolled single-date selection for CalendarRoot and DatePicker. */
export function useSingleCalendarSelection(
  options: SingleCalendarSelectionOptions,
): SingleCalendarSelection {
  const state = useControllableState<PlainDate | null>({
    value: () => {
      const value = options.value();
      return value === undefined ? undefined : normalizePlainDate(value);
    },
    defaultValue: () => normalizePlainDate(options.defaultValue()),
    equals: isSameDay,
    onChange: (value) => options.onUpdate(value),
  });
  const value = computed(() => state.value.value);

  function setValue(next: PlainDate | null, event: Event | null = null): boolean {
    const normalized = normalizePlainDate(next);
    const previous = value.value;
    const changed = state.set(normalized);
    if (changed) options.onChange(normalized, previous, event);
    return changed;
  }

  return {
    value,
    setValue,
    reset: state.reset,
    adapter: {
      mode: "single",
      anchorDate: () => value.value,
      hasSelection: () => value.value !== null,
      daySelection: (date) => (isSameDay(date, value.value) ? selectedDay : unselected),
      select: (date, event) => {
        setValue(date, event);
        options.onSelect?.(date, event);
      },
      hover: () => undefined,
      cancel: () => false,
    },
  };
}
