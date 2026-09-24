import { computed } from "vue";
import type { ComputedRef, ShallowRef } from "vue";

import {
  deriveDeterministicId,
  useDeterministicId,
} from "../../foundations/id/deterministic-id.ts";
import { resolveLocale, useLocaleValue } from "../../i18n/locale/locale-runtime.ts";
import { useSegmentedField } from "../date-field/field-segment-runtime.ts";
import {
  emptySegmentValues,
  resolveDayPeriodLabels,
  resolveHourCycle,
  resolveSegmentNames,
  resolveTimeSegmentLayout,
} from "../date-field/field-segments.ts";
import type {
  EditableSegmentType,
  FieldSegmentValues,
  SegmentHourCycle,
} from "../date-field/field-segments.ts";
import type { FieldSegmentPlaceholders } from "../date-field/date-field-types.ts";
import { compareTimes, formatIsoTime, isSameTime, normalizePlainTime } from "./plain-time.ts";
import type { HourCycle, PlainTime, TimeGranularity } from "./plain-time.ts";
import type { TimeFieldExpose, TimeFieldSlotState } from "./time-field-types.ts";

/** Props read by {@link useTimeField}; every key is present and may be undefined. */
export interface TimeFieldRuntimeProps {
  readonly id: string | null | undefined;
  readonly modelValue: PlainTime | null | undefined;
  readonly defaultValue: PlainTime | null | undefined;
  readonly min: PlainTime | null | undefined;
  readonly max: PlainTime | null | undefined;
  readonly hourCycle: HourCycle | undefined;
  readonly granularity: TimeGranularity | undefined;
  readonly placeholderValue: PlainTime | null | undefined;
  readonly locale: string | undefined;
  readonly dir: "ltr" | "rtl" | undefined;
  readonly disabled: boolean | undefined;
  readonly readOnly: boolean | undefined;
  readonly required: boolean | undefined;
  readonly ariaInvalid: boolean | undefined;
  readonly placeholders: FieldSegmentPlaceholders | undefined;
  readonly emptyText: string | undefined;
}

type TimeFieldEmit = {
  (event: "update:modelValue", value: PlainTime | null): void;
  (
    event: "change",
    value: PlainTime | null,
    previous: PlainTime | null,
    nativeEvent: Event | null,
  ): void;
};

type TimeFieldSetupExpose = {
  readonly [Key in keyof TimeFieldExpose]: TimeFieldExpose[Key] extends (
    ...args: never[]
  ) => unknown
    ? TimeFieldExpose[Key]
    : Key extends "root"
      ? Readonly<ShallowRef<HTMLDivElement | null>>
      : ComputedRef<TimeFieldExpose[Key]>;
};

function hourPadding(locale: string, cycle: SegmentHourCycle): number {
  try {
    const parts = new Intl.DateTimeFormat(locale, {
      timeStyle: "short",
      hourCycle: cycle,
      timeZone: "UTC",
    }).formatToParts(Date.UTC(2001, 0, 1, 9, 5));
    return parts.find((part) => part.type === "hour")?.value.length === 2 ? 2 : 1;
  } catch {
    return cycle === "h23" ? 2 : 1;
  }
}

/** Time-specific adapter over the shared segmented spinbutton engine. */
export function useTimeField(
  props: TimeFieldRuntimeProps,
  emit: TimeFieldEmit,
  root: Readonly<ShallowRef<HTMLDivElement | null>>,
) {
  const localeValue = useLocaleValue();
  const id = useDeterministicId({ id: () => props.id, hint: "time-field" });
  const locale = computed(() => resolveLocale(props.locale ?? localeValue.value.locale));
  const direction = computed<"ltr" | "rtl">(() =>
    props.dir === "rtl" || props.dir === "ltr" ? props.dir : localeValue.value.direction,
  );
  const cycle = computed<SegmentHourCycle>(() =>
    props.hourCycle === 12
      ? "h12"
      : props.hourCycle === 24
        ? "h23"
        : resolveHourCycle(locale.value),
  );
  const hourCycle = computed<HourCycle>(() => (cycle.value === "h12" ? 12 : 24));
  const granularity = computed<TimeGranularity>(() =>
    props.granularity === "hour" || props.granularity === "second" ? props.granularity : "minute",
  );
  const padding = computed(() => hourPadding(locale.value, cycle.value));
  const disabled = computed(() => props.disabled === true);
  const readOnly = computed(() => props.readOnly === true);
  const required = computed(() => props.required === true);

  function toValues(time: PlainTime): FieldSegmentValues {
    const twelve = cycle.value === "h12";
    return {
      ...emptySegmentValues,
      hour: twelve ? time.hour % 12 || 12 : time.hour,
      dayPeriod: twelve ? (time.hour >= 12 ? 1 : 0) : null,
      minute: time.minute,
      second: time.second,
    };
  }

  function fromValues(values: FieldSegmentValues): PlainTime | null {
    const twelve = cycle.value === "h12";
    if (values.hour === null || (twelve && values.dayPeriod === null)) return null;
    if (granularity.value !== "hour" && values.minute === null) return null;
    if (granularity.value === "second" && values.second === null) return null;
    const hour = twelve ? (values.hour % 12) + (values.dayPeriod === 1 ? 12 : 0) : values.hour;
    return normalizePlainTime({
      hour,
      minute: granularity.value === "hour" ? 0 : (values.minute ?? 0),
      second: granularity.value === "second" ? (values.second ?? 0) : 0,
    });
  }

  const field = useSegmentedField<PlainTime>({
    root,
    locale,
    direction,
    layout: computed(() => resolveTimeSegmentLayout(locale.value, cycle.value, granularity.value)),
    hourCycle: cycle,
    dayPeriodLabels: computed(() => resolveDayPeriodLabels(locale.value)),
    names: computed(() => resolveSegmentNames(locale.value)),
    placeholders: () => props.placeholders,
    emptyText: () => props.emptyText,
    monthName: (month) => String(month),
    padding: (type: EditableSegmentType) => (type === "hour" ? padding.value : 2),
    value: () => props.modelValue,
    defaultValue: () => props.defaultValue,
    normalize: (value) => normalizePlainTime(value),
    equals: isSameTime,
    toValues,
    fromValues,
    startValues: () =>
      toValues(normalizePlainTime(props.placeholderValue) ?? { hour: 0, minute: 0, second: 0 }),
    isInvalid: (value) => {
      const min = normalizePlainTime(props.min);
      const max = normalizePlainTime(props.max);
      return (
        (min !== null && compareTimes(value, min) < 0) ||
        (max !== null && compareTimes(value, max) > 0)
      );
    },
    disabled: () => disabled.value,
    readOnly: () => readOnly.value,
    onUpdate: (value) => emit("update:modelValue", value),
    onChange: (value, previous, event) => emit("change", value, previous, event),
  });

  const invalid = computed(() => field.invalid.value || props.ariaInvalid === true);
  const isoValue = computed(() =>
    field.value.value
      ? formatIsoTime(field.value.value, granularity.value === "second" ? "second" : "minute")
      : "",
  );
  const slotState = computed<TimeFieldSlotState>(() => ({
    value: field.value.value,
    segments: field.segments.value,
    state: field.fieldState.value,
    invalid: invalid.value,
    disabled: disabled.value,
    readOnly: readOnly.value,
    required: required.value,
    locale: locale.value,
    hourCycle: hourCycle.value,
    granularity: granularity.value,
  }));

  function segmentId(type: string): string {
    return deriveDeterministicId(id.value, type);
  }

  const exposed = {
    root,
    value: field.value,
    segments: field.segments,
    state: field.fieldState,
    invalid,
    disabled,
    readOnly,
    required,
    locale,
    hourCycle,
    granularity,
    focus: (segment?: EditableSegmentType, options?: FocusOptions) =>
      field.focusSegment(segment, options),
    setValue: field.setValue,
    clear: field.clear,
    reset: field.reset,
  } satisfies TimeFieldSetupExpose;

  return { field, id, direction, invalid, isoValue, slotState, segmentId, exposed };
}
