import { computed, shallowRef } from "vue";
import type { ComputedRef } from "vue";

import { useControllableState } from "../../foundations/controllable-state/controllable-state.ts";
import type {
  CalendarDaySelection,
  CalendarSelectionAdapter,
} from "../calendar/calendar-runtime.ts";
import {
  addDays,
  createDateRange,
  daysBetween,
  isDateInRange,
  isSameDay,
  isSameRange,
  normalizeDateRange,
} from "../calendar/plain-date.ts";
import type { DateMatcher, DateRange, PlainDate } from "../calendar/plain-date.ts";

/** Options for {@link useRangeCalendarSelection}. */
export interface RangeCalendarSelectionOptions {
  /** Controlled range; `undefined` selects uncontrolled mode. */
  readonly value: () => DateRange | null | undefined;
  /** Initial uncontrolled range. */
  readonly defaultValue: () => DateRange | null | undefined;
  /** Whether a committed range may span unavailable dates. */
  readonly allowNonContiguousRanges: () => boolean;
  /** Unavailable-date predicate used for contiguity checks. */
  readonly isDateUnavailable: () => DateMatcher | undefined;
  /** Called for every distinct requested range. */
  readonly onUpdate: (value: DateRange | null) => void;
  /** Called after a distinct user or imperative change. */
  readonly onChange: (
    value: DateRange | null,
    previous: DateRange | null,
    event: Event | null,
  ) => void;
  /** Called for every completed two-endpoint selection, even when unchanged. */
  readonly onSelect?: (value: DateRange, event: Event) => void;
  /** Called when the first endpoint of a new range is picked or cleared. */
  readonly onAnchorChange: (anchor: PlainDate | null) => void;
}

/** Range selection state plus its calendar adapter. */
export interface RangeCalendarSelection {
  readonly value: ComputedRef<DateRange | null>;
  readonly anchor: ComputedRef<PlainDate | null>;
  readonly adapter: CalendarSelectionAdapter;
  readonly setValue: (value: DateRange | null, event?: Event | null) => boolean;
  readonly reset: () => boolean;
}

/** Longest span scanned for unavailable dates before a range is committed. */
const maximumContiguityScanDays = 36_600;

/**
 * Two-click range selection: the first activation anchors the range, pointer
 * or focus movement previews it, and the second activation commits an ordered
 * {@link DateRange}. Escape cancels a pending anchor.
 */
export function useRangeCalendarSelection(
  options: RangeCalendarSelectionOptions,
): RangeCalendarSelection {
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
  const anchorState = shallowRef<PlainDate | null>(null);
  const hovered = shallowRef<PlainDate | null>(null);
  const anchor = computed(() => anchorState.value);
  const preview = computed(() =>
    anchorState.value
      ? createDateRange(anchorState.value, hovered.value ?? anchorState.value)
      : null,
  );

  function setAnchor(next: PlainDate | null): void {
    if (isSameDay(anchorState.value, next)) return;
    anchorState.value = next;
    hovered.value = next;
    options.onAnchorChange(next);
  }

  function setValue(next: DateRange | null, event: Event | null = null): boolean {
    const normalized = normalizeDateRange(next);
    const previous = value.value;
    const changed = state.set(normalized);
    if (changed) options.onChange(normalized, previous, event);
    return changed;
  }

  function spansUnavailable(range: DateRange): boolean {
    const unavailable = options.isDateUnavailable();
    if (!unavailable || options.allowNonContiguousRanges()) return false;
    const length = Math.min(daysBetween(range.start, range.end), maximumContiguityScanDays);
    for (let offset = 0; offset <= length; offset += 1) {
      if (unavailable(addDays(range.start, offset))) return true;
    }
    return false;
  }

  function daySelection(date: PlainDate): CalendarDaySelection {
    const range = preview.value ?? value.value;
    if (!range) {
      return {
        selected: false,
        rangeStart: false,
        rangeEnd: false,
        inRange: false,
        preview: false,
      };
    }
    const rangeStart = isSameDay(date, range.start);
    const rangeEnd = isSameDay(date, range.end);
    return {
      selected: rangeStart || rangeEnd,
      rangeStart,
      rangeEnd,
      inRange: isDateInRange(date, range),
      preview: preview.value !== null,
    };
  }

  function select(date: PlainDate, event: Event): void {
    const start = anchorState.value;
    if (!start) {
      setAnchor(date);
      return;
    }
    const range = createDateRange(start, date);
    if (spansUnavailable(range)) {
      setAnchor(date);
      return;
    }
    setAnchor(null);
    setValue(range, event);
    options.onSelect?.(range, event);
  }

  return {
    value,
    anchor,
    setValue,
    reset: () => {
      setAnchor(null);
      return state.reset();
    },
    adapter: {
      mode: "range",
      anchorDate: () => anchorState.value ?? value.value?.start ?? null,
      hasSelection: () => value.value !== null,
      daySelection,
      select,
      hover: (date) => {
        if (anchorState.value) hovered.value = date;
      },
      cancel: () => {
        if (!anchorState.value) return false;
        setAnchor(null);
        return true;
      },
    },
  };
}
