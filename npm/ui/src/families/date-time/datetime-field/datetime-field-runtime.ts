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
  resolveDateTimeSegmentLayout,
} from "../date-field/field-segments.ts";
import type {
  EditableSegmentType,
  FieldSegmentValues,
  SegmentHourCycle,
} from "../date-field/field-segments.ts";
import type { FieldSegmentPlaceholders } from "../date-field/date-field-types.ts";
import { createCalendarFormatters } from "../calendar/calendar-locale.ts";
import type { DateMatcher } from "../calendar/plain-date.ts";
import { hourPadding } from "../date-field/field-segment-runtime.ts";
import type { HourCycle, TimeGranularity } from "../time-field/plain-time.ts";
import {
  compareDateTimes,
  formatIsoDateTime,
  fromEpochMillisecondsDateTime,
  isSameDateTime,
  normalizePlainDateTime,
} from "./plain-date-time.ts";
import type { PlainDateTime } from "./plain-date-time.ts";
import type { DateTimeFieldExpose, DateTimeFieldSlotState } from "./datetime-field-types.ts";

/** Props read by {@link useDateTimeField}; every key is present and may be undefined. */
export interface DateTimeFieldRuntimeProps {
  readonly id: string | null | undefined;
  readonly modelValue: PlainDateTime | null | undefined;
  readonly defaultValue: PlainDateTime | null | undefined;
  readonly min: PlainDateTime | null | undefined;
  readonly max: PlainDateTime | null | undefined;
  readonly isDateUnavailable: DateMatcher | undefined;
  readonly timeZone: string | undefined;
  readonly hourCycle: HourCycle | undefined;
  readonly granularity: TimeGranularity | undefined;
  readonly placeholderValue: PlainDateTime | null | undefined;
  readonly locale: string | undefined;
  readonly dir: "ltr" | "rtl" | undefined;
  readonly disabled: boolean | undefined;
  readonly readOnly: boolean | undefined;
  readonly required: boolean | undefined;
  readonly ariaInvalid: boolean | undefined;
  readonly placeholders: FieldSegmentPlaceholders | undefined;
  readonly emptyText: string | undefined;
}

type DateTimeFieldEmit = {
  (event: "update:modelValue", value: PlainDateTime | null): void;
  (
    event: "change",
    value: PlainDateTime | null,
    previous: PlainDateTime | null,
    nativeEvent: Event | null,
  ): void;
};

type DateTimeFieldSetupExpose = {
  readonly [Key in keyof DateTimeFieldExpose]: DateTimeFieldExpose[Key] extends (
    ...args: never[]
  ) => unknown
    ? DateTimeFieldExpose[Key]
    : Key extends "root"
      ? Readonly<ShallowRef<HTMLDivElement | null>>
      : ComputedRef<DateTimeFieldExpose[Key]>;
};

function hostDateTime(timeZone: string | undefined): PlainDateTime | null {
  let zone = timeZone;
  if (!zone) {
    try {
      zone = new Intl.DateTimeFormat().resolvedOptions().timeZone;
    } catch {
      zone = "UTC";
    }
  }
  return fromEpochMillisecondsDateTime(Date.now(), zone);
}

/** Date-time adapter over the shared segmented spinbutton engine. */
export function useDateTimeField(
  props: DateTimeFieldRuntimeProps,
  emit: DateTimeFieldEmit,
  root: Readonly<ShallowRef<HTMLDivElement | null>>,
) {
  const localeValue = useLocaleValue();
  const id = useDeterministicId({ id: () => props.id, hint: "datetime-field" });
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
  const formatters = computed(() => createCalendarFormatters({ locale: locale.value }));
  const disabled = computed(() => props.disabled === true);
  const readOnly = computed(() => props.readOnly === true);
  const required = computed(() => props.required === true);

  function toValues(time: PlainDateTime): FieldSegmentValues {
    const twelve = cycle.value === "h12";
    return {
      ...emptySegmentValues,
      year: time.year,
      month: time.month,
      day: time.day,
      hour: twelve ? time.hour % 12 || 12 : time.hour,
      dayPeriod: twelve ? (time.hour >= 12 ? 1 : 0) : null,
      minute: time.minute,
      second: time.second,
    };
  }

  function fromValues(values: FieldSegmentValues): PlainDateTime | null {
    const twelve = cycle.value === "h12";
    if (values.year === null || values.month === null || values.day === null) return null;
    if (values.hour === null || (twelve && values.dayPeriod === null)) return null;
    if (granularity.value !== "hour" && values.minute === null) return null;
    if (granularity.value === "second" && values.second === null) return null;
    const hour = twelve ? (values.hour % 12) + (values.dayPeriod === 1 ? 12 : 0) : values.hour;
    return normalizePlainDateTime({
      year: values.year,
      month: values.month,
      day: values.day,
      hour,
      minute: granularity.value === "hour" ? 0 : (values.minute ?? 0),
      second: granularity.value === "second" ? (values.second ?? 0) : 0,
    });
  }

  const field = useSegmentedField<PlainDateTime>({
    root,
    locale,
    direction,
    layout: computed(() =>
      resolveDateTimeSegmentLayout(locale.value, cycle.value, granularity.value),
    ),
    hourCycle: cycle,
    dayPeriodLabels: computed(() => resolveDayPeriodLabels(locale.value)),
    names: computed(() => resolveSegmentNames(locale.value)),
    placeholders: () => props.placeholders,
    emptyText: () => props.emptyText,
    monthName: (month) => formatters.value.monthName(month),
    padding: (type: EditableSegmentType) =>
      type === "hour" ? padding.value : type === "year" ? 1 : 2,
    value: () => props.modelValue,
    defaultValue: () => props.defaultValue,
    normalize: (value) => normalizePlainDateTime(value),
    equals: isSameDateTime,
    toValues,
    fromValues,
    startValues: () => {
      const start = normalizePlainDateTime(props.placeholderValue) ?? hostDateTime(props.timeZone);
      return start ? toValues({ ...start, minute: 0, second: 0 }) : emptySegmentValues;
    },
    isInvalid: (value) => {
      const min = normalizePlainDateTime(props.min);
      const max = normalizePlainDateTime(props.max);
      return (
        (min !== null && compareDateTimes(value, min) < 0) ||
        (max !== null && compareDateTimes(value, max) > 0) ||
        props.isDateUnavailable?.(value) === true
      );
    },
    disabled: () => disabled.value,
    readOnly: () => readOnly.value,
    onUpdate: (value) => emit("update:modelValue", value),
    onChange: (value, previous, event) => emit("change", value, previous, event),
  });

  const invalid = computed(() => field.invalid.value || props.ariaInvalid === true);
  const isoValue = computed(() =>
    field.value.value ? formatIsoDateTime(field.value.value, granularity.value) : "",
  );
  const slotState = computed<DateTimeFieldSlotState>(() => ({
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
  } satisfies DateTimeFieldSetupExpose;

  return { field, id, direction, invalid, isoValue, slotState, segmentId, exposed };
}
