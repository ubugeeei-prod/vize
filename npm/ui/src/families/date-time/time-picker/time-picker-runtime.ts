import { computed } from "vue";

import { useControllableState } from "../../foundations/controllable-state/controllable-state.ts";
import { resolveHourCycle } from "../date-field/field-segments.ts";
import {
  compareTimes,
  formatIsoTime,
  isSameTime,
  normalizePlainTime,
  parseIsoTime,
  toSecondOfDay,
} from "../time-field/plain-time.ts";
import type { HourCycle, PlainTime } from "../time-field/plain-time.ts";
import type { TimePickerSlot } from "./time-picker-types.ts";

/** Options for {@link createTimeSlots}. */
export interface TimeSlotOptions {
  /** First slot, inclusive. @default 00:00 */
  readonly min?: PlainTime | null | undefined;
  /** Last possible slot, inclusive. @default 23:59 */
  readonly max?: PlainTime | null | undefined;
  /** Minutes between slots, clamped to 1–1440. @default 30 */
  readonly step?: number | undefined;
}

/** Generate slot times from `min` to `max` (inclusive) every `step` minutes. */
export function createTimeSlots(options: TimeSlotOptions = {}): readonly PlainTime[] {
  const min = normalizePlainTime(options.min) ?? { hour: 0, minute: 0, second: 0 };
  const max = normalizePlainTime(options.max) ?? { hour: 23, minute: 59, second: 59 };
  const step = Math.min(1_440, Math.max(1, Math.trunc(options.step ?? 30) || 30));
  const first = Math.ceil(toSecondOfDay(min) / 60);
  const last = Math.floor(toSecondOfDay(max) / 60);
  const slots: PlainTime[] = [];
  for (let minute = first; minute <= last; minute += step) {
    slots.push(Object.freeze({ hour: Math.floor(minute / 60), minute: minute % 60, second: 0 }));
  }
  return Object.freeze(slots);
}

/** Inputs of {@link useTimePicker}. */
export interface TimePickerInput {
  readonly value: () => PlainTime | null | undefined;
  readonly defaultValue: () => PlainTime | null | undefined;
  readonly min: () => PlainTime | null | undefined;
  readonly max: () => PlainTime | null | undefined;
  readonly step: () => number | undefined;
  readonly hourCycle: () => HourCycle | undefined;
  readonly locale: () => string;
  readonly isTimeUnavailable: () => ((time: PlainTime) => boolean) | undefined;
  readonly readOnly: () => boolean;
  readonly onUpdate: (value: PlainTime | null) => void;
  readonly onChange: (
    value: PlainTime | null,
    previous: PlainTime | null,
    event: Event | null,
  ) => void;
}

/** Slot generation, labels, and value mapping behind TimePicker. */
export function useTimePicker(input: TimePickerInput) {
  const state = useControllableState<PlainTime | null>({
    value: () => {
      const value = input.value();
      return value === undefined ? undefined : normalizePlainTime(value);
    },
    defaultValue: () => normalizePlainTime(input.defaultValue()),
    equals: isSameTime,
    onChange: (value) => input.onUpdate(value),
  });
  const value = computed(() => state.value.value);
  const hourCycle = computed<HourCycle>(() => {
    const requested = input.hourCycle();
    if (requested === 12 || requested === 24) return requested;
    return resolveHourCycle(input.locale()) === "h12" ? 12 : 24;
  });
  const formatter = computed(() => {
    const options: Intl.DateTimeFormatOptions = {
      timeStyle: "short",
      hourCycle: hourCycle.value === 12 ? "h12" : "h23",
      timeZone: "UTC",
    };
    try {
      return new Intl.DateTimeFormat(input.locale(), options);
    } catch {
      return new Intl.DateTimeFormat("en-US", options);
    }
  });
  const slots = computed<readonly TimePickerSlot[]>(() =>
    createTimeSlots({ min: input.min(), max: input.max(), step: input.step() }).map((time) => ({
      time,
      value: formatIsoTime(time),
      label: formatter.value.format(Date.UTC(2001, 0, 1, time.hour, time.minute)),
      disabled: input.isTimeUnavailable()?.(time) === true,
      selected: value.value !== null && compareTimes(value.value, time) === 0,
    })),
  );
  const listboxValue = computed(() => {
    const selected = slots.value.find((slot) => slot.selected);
    return selected ? selected.value : null;
  });

  function setValue(next: PlainTime | null, event: Event | null = null): boolean {
    const normalized = normalizePlainTime(next);
    const previous = value.value;
    const changed = state.set(normalized);
    if (changed) input.onChange(normalized, previous, event);
    return changed;
  }

  function onListboxChange(next: string | readonly string[] | null, event: Event | null): void {
    if (input.readOnly() || Array.isArray(next)) return;
    const time = typeof next === "string" ? parseIsoTime(next) : null;
    setValue(time, event);
  }

  return { value, hourCycle, slots, listboxValue, setValue, onListboxChange };
}
