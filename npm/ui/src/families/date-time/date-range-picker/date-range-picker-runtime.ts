import { computed, shallowRef, watch } from "vue";

import { useControllableState } from "../../foundations/controllable-state/controllable-state.ts";
import {
  createDateRange,
  isSameDay,
  isSameRange,
  normalizeDateRange,
  normalizePlainDate,
} from "../calendar/plain-date.ts";
import type { DateRange, PlainDate } from "../calendar/plain-date.ts";
import type { DateRangeBoundary, DateRangeBoundaryRecord } from "./date-range-picker-context.ts";

/** Inputs of {@link useRangePickerValue}. */
export interface RangePickerValueInput {
  readonly value: () => DateRange | null | undefined;
  readonly defaultValue: () => DateRange | null | undefined;
  readonly locked: () => boolean;
  readonly onUpdate: (value: DateRange | null) => void;
  readonly onChange: (
    value: DateRange | null,
    previous: DateRange | null,
    event: Event | null,
  ) => void;
}

/**
 * Range value plus per-endpoint drafts, so one field can hold a date while
 * the other is still empty. The committed value is `null` until both
 * endpoints are known, then an ordered {@link DateRange}.
 */
export function useRangePickerValue(input: RangePickerValueInput) {
  const state = useControllableState<DateRange | null>({
    value: () => {
      const value = input.value();
      return value === undefined ? undefined : normalizeDateRange(value);
    },
    defaultValue: () => normalizeDateRange(input.defaultValue()),
    equals: isSameRange,
    onChange: (value) => input.onUpdate(value),
  });
  const value = computed(() => state.value.value);
  const start = shallowRef<PlainDate | null>(value.value?.start ?? null);
  const end = shallowRef<PlainDate | null>(value.value?.end ?? null);
  const drafts = computed<DateRangeBoundaryRecord<PlainDate | null>>(() => ({
    start: start.value,
    end: end.value,
  }));

  watch(value, (next) => {
    if (next) {
      start.value = next.start;
      end.value = next.end;
    } else if (start.value && end.value) {
      start.value = null;
      end.value = null;
    }
  });

  function commit(next: DateRange | null, event: Event | null): boolean {
    const previous = value.value;
    const changed = state.set(next);
    if (changed) input.onChange(next, previous, event);
    return changed;
  }

  function setValue(next: DateRange | null, event: Event | null = null): boolean {
    if (input.locked()) return false;
    const normalized = normalizeDateRange(next);
    start.value = normalized?.start ?? null;
    end.value = normalized?.end ?? null;
    return commit(normalized, event);
  }

  function setBoundary(
    boundary: DateRangeBoundary,
    date: PlainDate | null,
    event: Event | null = null,
  ): void {
    if (input.locked()) return;
    const normalized = normalizePlainDate(date);
    const target = boundary === "start" ? start : end;
    if (isSameDay(target.value, normalized)) return;
    target.value = normalized;
    commit(start.value && end.value ? createDateRange(start.value, end.value) : null, event);
  }

  return { value, drafts, setValue, setBoundary };
}
