import { computed, shallowRef } from "vue";
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

/** Options for {@link useMultipleCalendarSelection}. */
export interface MultipleCalendarSelectionOptions {
  /** Controlled dates; `undefined` selects uncontrolled mode. */
  readonly value: () => readonly PlainDate[] | undefined;
  /** Initial uncontrolled dates. */
  readonly defaultValue: () => readonly PlainDate[] | undefined;
  /** Largest number of selected dates; further activations are ignored. */
  readonly maxSelections: () => number | undefined;
  /** Called for every distinct requested value. */
  readonly onUpdate: (value: readonly PlainDate[]) => void;
  /** Called after a distinct user or imperative change. */
  readonly onChange: (
    value: readonly PlainDate[],
    previous: readonly PlainDate[],
    event: Event | null,
  ) => void;
  /** Called for every user toggle with the date and whether it is now selected. */
  readonly onToggle?: (date: PlainDate, selected: boolean, event: Event) => void;
}

/** Multiple-date selection state plus its calendar adapter. */
export interface MultipleCalendarSelection {
  readonly value: ComputedRef<readonly PlainDate[]>;
  readonly adapter: CalendarSelectionAdapter;
  readonly setValue: (value: readonly PlainDate[], event?: Event | null) => boolean;
  readonly toggle: (date: PlainDate, event?: Event | null) => boolean;
  readonly reset: () => boolean;
}

/** Sort, de-duplicate, validate, and freeze a list of dates. */
export function normalizeDateList(
  value: readonly PlainDate[] | null | undefined,
): readonly PlainDate[] {
  if (!Array.isArray(value)) return Object.freeze([]);
  const byIso = new Map<number, PlainDate>();
  for (const candidate of value) {
    const date = normalizePlainDate(candidate);
    if (date) byIso.set(date.year * 10_000 + date.month * 100 + date.day, date);
  }
  return Object.freeze([...byIso.entries()].sort(([a], [b]) => a - b).map(([, date]) => date));
}

function sameList(left: readonly PlainDate[], right: readonly PlainDate[]): boolean {
  return left.length === right.length && left.every((date, index) => isSameDay(date, right[index]));
}

/** Controlled/uncontrolled multiple-date selection that toggles dates on activation. */
export function useMultipleCalendarSelection(
  options: MultipleCalendarSelectionOptions,
): MultipleCalendarSelection {
  const state = useControllableState<readonly PlainDate[]>({
    value: () => {
      const value = options.value();
      return value === undefined ? undefined : normalizeDateList(value);
    },
    defaultValue: () => normalizeDateList(options.defaultValue()),
    equals: sameList,
    onChange: (value) => options.onUpdate(value),
  });
  const value = computed(() => state.value.value);
  const lastToggled = shallowRef<PlainDate | null>(null);

  function setValue(next: readonly PlainDate[], event: Event | null = null): boolean {
    const normalized = normalizeDateList(next);
    const previous = value.value;
    const changed = state.set(normalized);
    if (changed) options.onChange(normalized, previous, event);
    return changed;
  }

  function toggle(date: PlainDate, event: Event | null = null): boolean {
    const normalized = normalizePlainDate(date);
    if (!normalized) return false;
    const selected = value.value.some((candidate) => isSameDay(candidate, normalized));
    const limit = options.maxSelections();
    if (!selected && limit !== undefined && value.value.length >= Math.max(0, Math.trunc(limit))) {
      return false;
    }
    lastToggled.value = normalized;
    return setValue(
      selected
        ? value.value.filter((candidate) => !isSameDay(candidate, normalized))
        : [...value.value, normalized],
      event,
    );
  }

  return {
    value,
    setValue,
    toggle,
    reset: state.reset,
    adapter: {
      mode: "multiple",
      anchorDate: () => lastToggled.value ?? value.value[0] ?? null,
      hasSelection: () => value.value.length > 0,
      daySelection: (date) =>
        value.value.some((candidate) => isSameDay(candidate, date)) ? selectedDay : unselected,
      select: (date, event) => {
        const wasSelected = value.value.some((candidate) => isSameDay(candidate, date));
        if (toggle(date, event)) options.onToggle?.(date, !wasSelected, event);
      },
      hover: () => undefined,
      cancel: () => false,
    },
  };
}
