import { computed } from "vue";
import type { ComputedRef, ShallowRef } from "vue";

import {
  deriveDeterministicId,
  useDeterministicId,
} from "../../foundations/id/deterministic-id.ts";
import { resolveLocale, useLocaleValue } from "../../i18n/locale/locale-runtime.ts";
import { createCalendarFormatters } from "../calendar/calendar-locale.ts";
import {
  compareDates,
  formatIsoDate,
  fromEpochMilliseconds,
  isDateWithin,
  isSameDay,
  normalizePlainDate,
} from "../calendar/plain-date.ts";
import type { DateMatcher, PlainDate } from "../calendar/plain-date.ts";
import type {
  DateFieldExpose,
  DateFieldSlotState,
  FieldSegmentPlaceholders,
} from "./date-field-types.ts";
import { useSegmentedField } from "./field-segment-runtime.ts";
import {
  emptySegmentValues,
  resolveDateSegmentLayout,
  resolveSegmentNames,
} from "./field-segments.ts";
import type { EditableSegmentType, FieldSegmentValues } from "./field-segments.ts";

/** Props read by {@link useDateField}; every key is present and may be undefined. */
export interface DateFieldRuntimeProps {
  readonly id: string | null | undefined;
  readonly modelValue: PlainDate | null | undefined;
  readonly defaultValue: PlainDate | null | undefined;
  readonly min: PlainDate | null | undefined;
  readonly max: PlainDate | null | undefined;
  readonly isDateUnavailable: DateMatcher | undefined;
  readonly placeholderValue: PlainDate | null | undefined;
  readonly timeZone: string | undefined;
  readonly locale: string | undefined;
  readonly dir: "ltr" | "rtl" | undefined;
  readonly disabled: boolean | undefined;
  readonly readOnly: boolean | undefined;
  readonly required: boolean | undefined;
  readonly ariaInvalid: boolean | undefined;
  readonly placeholders: FieldSegmentPlaceholders | undefined;
  readonly emptyText: string | undefined;
}

type DateFieldEmit = {
  (event: "update:modelValue", value: PlainDate | null): void;
  (
    event: "change",
    value: PlainDate | null,
    previous: PlainDate | null,
    nativeEvent: Event | null,
  ): void;
};

type DateFieldSetupExpose = {
  readonly [Key in keyof DateFieldExpose]: DateFieldExpose[Key] extends (
    ...args: never[]
  ) => unknown
    ? DateFieldExpose[Key]
    : Key extends "root"
      ? Readonly<ShallowRef<HTMLDivElement | null>>
      : ComputedRef<DateFieldExpose[Key]>;
};

function hostDate(timeZone: string | undefined): PlainDate | null {
  let zone = timeZone;
  if (!zone) {
    try {
      zone = new Intl.DateTimeFormat().resolvedOptions().timeZone;
    } catch {
      zone = "UTC";
    }
  }
  return fromEpochMilliseconds(Date.now(), zone) ?? fromEpochMilliseconds(Date.now(), "UTC");
}

/** Date-specific adapter over the shared segmented spinbutton engine. */
export function useDateField(
  props: DateFieldRuntimeProps,
  emit: DateFieldEmit,
  root: Readonly<ShallowRef<HTMLDivElement | null>>,
) {
  const localeValue = useLocaleValue();
  const id = useDeterministicId({ id: () => props.id, hint: "date-field" });
  const locale = computed(() => resolveLocale(props.locale ?? localeValue.value.locale));
  const direction = computed<"ltr" | "rtl">(() =>
    props.dir === "rtl" || props.dir === "ltr" ? props.dir : localeValue.value.direction,
  );
  const formatters = computed(() => createCalendarFormatters({ locale: locale.value }));
  const bounds = computed(() => {
    const min = normalizePlainDate(props.min);
    const max = normalizePlainDate(props.max);
    return min && max && compareDates(min, max) > 0 ? { min: max, max: min } : { min, max };
  });
  const disabled = computed(() => props.disabled === true);
  const readOnly = computed(() => props.readOnly === true);
  const required = computed(() => props.required === true);

  const field = useSegmentedField<PlainDate>({
    root,
    locale,
    direction,
    layout: computed(() => resolveDateSegmentLayout(locale.value)),
    hourCycle: computed(() => "h23"),
    dayPeriodLabels: computed(() => ["AM", "PM"]),
    names: computed(() => resolveSegmentNames(locale.value)),
    placeholders: () => props.placeholders,
    emptyText: () => props.emptyText,
    monthName: (month) => formatters.value.monthName(month),
    padding: (type: EditableSegmentType) => (type === "year" ? 1 : 2),
    value: () => props.modelValue,
    defaultValue: () => props.defaultValue,
    normalize: (value) => normalizePlainDate(value),
    equals: isSameDay,
    toValues: (value): FieldSegmentValues => ({
      ...emptySegmentValues,
      year: value.year,
      month: value.month,
      day: value.day,
    }),
    fromValues: (values) =>
      values.year === null || values.month === null || values.day === null
        ? null
        : normalizePlainDate({ year: values.year, month: values.month, day: values.day }),
    startValues: () => {
      const start = normalizePlainDate(props.placeholderValue) ?? hostDate(props.timeZone);
      return start
        ? { ...emptySegmentValues, year: start.year, month: start.month, day: start.day }
        : emptySegmentValues;
    },
    isInvalid: (value) =>
      !isDateWithin(value, bounds.value.min, bounds.value.max) ||
      props.isDateUnavailable?.(value) === true,
    disabled: () => disabled.value,
    readOnly: () => readOnly.value,
    onUpdate: (value) => emit("update:modelValue", value),
    onChange: (value, previous, event) => emit("change", value, previous, event),
  });

  const invalid = computed(() => field.invalid.value || props.ariaInvalid === true);
  const isoValue = computed(() => (field.value.value ? formatIsoDate(field.value.value) : ""));
  const slotState = computed<DateFieldSlotState>(() => ({
    value: field.value.value,
    segments: field.segments.value,
    state: field.fieldState.value,
    invalid: invalid.value,
    disabled: disabled.value,
    readOnly: readOnly.value,
    required: required.value,
    locale: locale.value,
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
    focus: (segment?: EditableSegmentType, options?: FocusOptions) =>
      field.focusSegment(segment, options),
    setValue: field.setValue,
    clear: field.clear,
    reset: field.reset,
  } satisfies DateFieldSetupExpose;

  return { field, id, direction, invalid, isoValue, slotState, segmentId, exposed };
}
