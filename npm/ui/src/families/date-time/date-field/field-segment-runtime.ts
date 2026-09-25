import { computed, nextTick, shallowRef, watch } from "vue";
import type { ComputedRef, ShallowRef } from "vue";

import { useControllableState } from "../../foundations/controllable-state/controllable-state.ts";

import { daysInMonth } from "../calendar/plain-date.ts";

/** Editable date segments. */
export type DateSegmentType = "year" | "month" | "day";

/** Editable time segments. `dayPeriod` is AM/PM and stores `0` or `1`. */
export type TimeSegmentType = "hour" | "minute" | "second" | "dayPeriod";

/** Every editable segment type. */
export type EditableSegmentType = DateSegmentType | TimeSegmentType;

/** Every rendered segment type; literals are separators such as `/` or `:`. */
export type FieldSegmentType = EditableSegmentType | "literal";

/** Current value per editable segment; `null` means empty. */
export type FieldSegmentValues = Readonly<Record<EditableSegmentType, number | null>>;

/** One entry of a locale segment layout. */
export interface FieldSegmentLayoutPart {
  /** Segment type. */
  readonly type: FieldSegmentType;

  /** Literal text for separators; empty for editable segments. */
  readonly literal: string;
}

/** Hour clock used by time segments; `h12` shows 1–12 plus a day period. */
export type SegmentHourCycle = "h12" | "h23";

/** Inclusive numeric bounds of one editable segment. */
export interface FieldSegmentBounds {
  readonly min: number;
  readonly max: number;
}

/** Result of typing one digit into a segment. */
export interface SegmentDigitResult {
  /** New segment value; `null` while the typed prefix is below the minimum (for example a lone `0` month). */
  readonly value: number | null;

  /** Digits typed so far for the segment. */
  readonly buffer: string;

  /** Whether no further digit can extend the value, so focus should advance. */
  readonly advance: boolean;
}

/** Empty values for every editable segment. */
export const emptySegmentValues: FieldSegmentValues = Object.freeze({
  year: null,
  month: null,
  day: null,
  hour: null,
  minute: null,
  second: null,
  dayPeriod: null,
});

const editableTypes = new Set<string>([
  "year",
  "month",
  "day",
  "hour",
  "minute",
  "second",
  "dayPeriod",
]);

/** Whether a string names an editable segment type. */
export function isEditableSegmentType(value: string): value is EditableSegmentType {
  return editableTypes.has(value);
}

function collectLayout(parts: readonly Intl.DateTimeFormatPart[]): FieldSegmentLayoutPart[] {
  const layout: FieldSegmentLayoutPart[] = [];
  for (const part of parts) {
    if (isEditableSegmentType(part.type)) {
      if (!layout.some((entry) => entry.type === part.type)) {
        layout.push({ type: part.type, literal: "" });
      }
    } else if (part.type === "literal") {
      const previous = layout.at(-1);
      if (previous?.type === "literal") {
        layout[layout.length - 1] = { type: "literal", literal: previous.literal + part.value };
      } else {
        layout.push({ type: "literal", literal: part.value });
      }
    }
  }
  while (layout[0]?.type === "literal") layout.shift();
  while (layout.at(-1)?.type === "literal") layout.pop();
  return layout;
}

function safeFormatter(locale: string, options: Intl.DateTimeFormatOptions): Intl.DateTimeFormat {
  try {
    return new Intl.DateTimeFormat(locale, { ...options, timeZone: "UTC" });
  } catch {
    return new Intl.DateTimeFormat("en-US", { ...options, timeZone: "UTC" });
  }
}

/** Locale order and separators of year, month, and day segments. */
export function resolveDateSegmentLayout(locale: string): readonly FieldSegmentLayoutPart[] {
  const formatter = safeFormatter(locale, {
    calendar: "gregory",
    year: "numeric",
    month: "2-digit",
    day: "2-digit",
  });
  return collectLayout(formatter.formatToParts(Date.UTC(2001, 10, 22)));
}

/** Locale default hour cycle, normalized to `h12` or `h23`. */
export function resolveHourCycle(locale: string): SegmentHourCycle {
  const cycle = safeFormatter(locale, { hour: "numeric" }).resolvedOptions().hourCycle;
  return cycle === "h11" || cycle === "h12" ? "h12" : "h23";
}

/** Locale order and separators of hour, minute, second, and day-period segments. */
export function resolveTimeSegmentLayout(
  locale: string,
  hourCycle: SegmentHourCycle,
  granularity: "hour" | "minute" | "second",
): readonly FieldSegmentLayoutPart[] {
  const options: Intl.DateTimeFormatOptions = { hour: "2-digit", hourCycle };
  if (granularity !== "hour") options.minute = "2-digit";
  if (granularity === "second") options.second = "2-digit";
  return collectLayout(
    safeFormatter(locale, options).formatToParts(Date.UTC(2001, 0, 1, 21, 5, 9)),
  );
}

/** Locale order and separators of date and time segments together. */
export function resolveDateTimeSegmentLayout(
  locale: string,
  hourCycle: SegmentHourCycle,
  granularity: "hour" | "minute" | "second",
): readonly FieldSegmentLayoutPart[] {
  const options: Intl.DateTimeFormatOptions = {
    calendar: "gregory",
    year: "numeric",
    month: "2-digit",
    day: "2-digit",
    hour: "2-digit",
    hourCycle,
  };
  if (granularity !== "hour") options.minute = "2-digit";
  if (granularity === "second") options.second = "2-digit";
  return collectLayout(
    safeFormatter(locale, options).formatToParts(Date.UTC(2001, 10, 22, 21, 5, 9)),
  );
}

/** Digits the locale uses for the hour at 9 o'clock (`2` pads to `09`). */
export function hourPadding(locale: string, cycle: SegmentHourCycle): number {
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

/** Localized AM and PM labels in that order. */
export function resolveDayPeriodLabels(locale: string): readonly [string, string] {
  const formatter = safeFormatter(locale, { hour: "numeric", hourCycle: "h12" });
  const read = (hour: number) =>
    formatter.formatToParts(Date.UTC(2001, 0, 1, hour)).find((part) => part.type === "dayPeriod")
      ?.value ?? (hour < 12 ? "AM" : "PM");
  return [read(9), read(21)];
}

/** Localized segment field names from `Intl.DisplayNames`, with English fallbacks. */
export function resolveSegmentNames(locale: string): Readonly<Record<EditableSegmentType, string>> {
  const fallback: Record<EditableSegmentType, string> = {
    year: "year",
    month: "month",
    day: "day",
    hour: "hour",
    minute: "minute",
    second: "second",
    dayPeriod: "AM/PM",
  };
  try {
    const names = new Intl.DisplayNames(locale, { type: "dateTimeField" });
    for (const type of Object.keys(fallback) as EditableSegmentType[]) {
      const name = names.of(type);
      if (name) fallback[type] = name;
    }
  } catch {
    // Engines without dateTimeField display names keep English labels.
  }
  return fallback;
}

/** Numeric bounds of a segment given the other segment values. */
export function segmentBounds(
  type: EditableSegmentType,
  values: FieldSegmentValues,
  hourCycle: SegmentHourCycle = "h23",
): FieldSegmentBounds {
  switch (type) {
    case "year":
      return { min: 1, max: 9_999 };
    case "month":
      return { min: 1, max: 12 };
    case "day":
      return {
        min: 1,
        max: values.month === null ? 31 : daysInMonth(values.year ?? 2000, values.month),
      };
    case "hour":
      return hourCycle === "h12" ? { min: 1, max: 12 } : { min: 0, max: 23 };
    case "dayPeriod":
      return { min: 0, max: 1 };
    default:
      return { min: 0, max: 59 };
  }
}

/** Maximum digits typed into a segment before focus advances. */
export function segmentMaxDigits(type: EditableSegmentType): number {
  if (type === "year") return 4;
  return type === "dayPeriod" ? 1 : 2;
}

/** Default PageUp/PageDown step per segment. */
export function segmentPageStep(type: EditableSegmentType): number {
  switch (type) {
    case "year":
      return 10;
    case "month":
      return 3;
    case "day":
      return 7;
    case "hour":
      return 2;
    case "dayPeriod":
      return 1;
    default:
      return 15;
  }
}

/** Step a segment by `delta`, wrapping within bounds; empty segments start at `start`. */
export function stepSegmentValue(
  value: number | null,
  delta: number,
  bounds: FieldSegmentBounds,
  start: number,
): number {
  if (value === null) return Math.min(bounds.max, Math.max(bounds.min, start));
  const span = bounds.max - bounds.min + 1;
  const offset = (((value - bounds.min + delta) % span) + span) % span;
  return bounds.min + offset;
}

/** Type one ASCII digit into a segment buffer. */
export function typeSegmentDigit(
  buffer: string,
  digit: string,
  bounds: FieldSegmentBounds,
  maxDigits: number,
): SegmentDigitResult {
  let nextBuffer = `${buffer}${digit}`.slice(-maxDigits);
  let next = Number(nextBuffer);
  if (next > bounds.max) {
    nextBuffer = digit;
    next = Number(digit);
  }
  const complete = nextBuffer.length >= maxDigits || next * 10 > bounds.max;
  const value = next >= bounds.min && next <= bounds.max ? next : null;
  return { value, buffer: nextBuffer, advance: complete && value !== null };
}

/** Remove the last digit of a segment value; single-digit values become empty. */
export function backspaceSegmentValue(
  value: number | null,
  bounds: FieldSegmentBounds,
): number | null {
  if (value === null) return null;
  const next = Math.trunc(value / 10);
  return next === 0 || next < bounds.min ? null : next;
}

/** Value-state token mirrored to a segmented field's `data-state`. */
export type SegmentedFieldState = "complete" | "empty" | "invalid" | "partial";

/** State of one rendered segment. */
export interface FieldSegmentState {
  /** Segment type; `literal` for separators. */
  readonly type: FieldSegmentType;

  /** Zero-based position in the layout. */
  readonly index: number;

  /** Visible text: the localized value, a typed prefix, a placeholder, or a literal. */
  readonly text: string;

  /** Current numeric value, or `null` when empty or literal. */
  readonly value: number | null;

  /** Lowest accepted value. */
  readonly min: number;

  /** Highest accepted value. */
  readonly max: number;

  /** Whether the segment shows its placeholder. */
  readonly placeholder: boolean;

  /** Whether the segment is an editable spinbutton. */
  readonly editable: boolean;

  /** Localized field name used as the accessible label. */
  readonly label: string;

  /** Text announced as `aria-valuetext`. */
  readonly valueText: string;
}

/** Inputs of {@link useSegmentedField}. */
export interface SegmentedFieldOptions<Value> {
  readonly root: Readonly<ShallowRef<HTMLElement | null>>;
  readonly locale: ComputedRef<string>;
  readonly direction: ComputedRef<"ltr" | "rtl">;
  readonly layout: ComputedRef<readonly FieldSegmentLayoutPart[]>;
  readonly hourCycle: ComputedRef<SegmentHourCycle>;
  readonly dayPeriodLabels: ComputedRef<readonly [string, string]>;
  readonly names: ComputedRef<Readonly<Record<EditableSegmentType, string>>>;
  readonly placeholders: () => Partial<Record<EditableSegmentType, string>> | undefined;
  readonly emptyText: () => string | undefined;
  readonly monthName: (month: number) => string;
  readonly padding: (type: EditableSegmentType) => number;
  readonly value: () => Value | null | undefined;
  readonly defaultValue: () => Value | null | undefined;
  readonly normalize: (value: Value | null | undefined) => Value | null;
  readonly equals: (left: Value | null, right: Value | null) => boolean;
  readonly toValues: (value: Value) => FieldSegmentValues;
  readonly fromValues: (values: FieldSegmentValues) => Value | null;
  readonly startValues: () => FieldSegmentValues;
  readonly isInvalid: (value: Value) => boolean;
  readonly disabled: () => boolean;
  readonly readOnly: () => boolean;
  readonly onUpdate: (value: Value | null) => void;
  readonly onChange: (value: Value | null, previous: Value | null, event: Event | null) => void;
}

const defaultPlaceholders: Readonly<Record<EditableSegmentType, string>> = {
  year: "yyyy",
  month: "mm",
  day: "dd",
  hour: "––",
  minute: "––",
  second: "––",
  dayPeriod: "––",
};

/**
 * Shared segmented spinbutton engine for DateField and TimeField.
 *
 * Segment values stay editable while incomplete; a complete, valid set of
 * segments commits a value, and clearing any segment commits `null`.
 */
export function useSegmentedField<Value>(options: SegmentedFieldOptions<Value>) {
  const state = useControllableState<Value | null>({
    value: () => {
      const value = options.value();
      return value === undefined ? undefined : options.normalize(value);
    },
    defaultValue: () => options.normalize(options.defaultValue()),
    equals: options.equals,
    onChange: (value) => options.onUpdate(value),
  });
  const value = computed(() => state.value.value);
  const valuesFor = (next: Value | null) =>
    next === null ? emptySegmentValues : options.toValues(next);
  const values = shallowRef<FieldSegmentValues>(valuesFor(value.value));
  const buffer = shallowRef("");
  const activeSegment = shallowRef<EditableSegmentType | null>(null);
  const numberFormatters = computed(() => {
    const cache = new Map<number, Intl.NumberFormat>();
    const locale = options.locale.value;
    return (padding: number) => {
      let formatter = cache.get(padding);
      if (!formatter) {
        formatter = new Intl.NumberFormat(locale, {
          useGrouping: false,
          minimumIntegerDigits: Math.max(1, padding),
        });
        cache.set(padding, formatter);
      }
      return formatter;
    };
  });
  const editableTypes = computed(() =>
    options.layout.value
      .map((part) => part.type)
      .filter((type): type is EditableSegmentType => type !== "literal"),
  );

  watch(value, (next) => {
    if (!options.equals(next, options.fromValues(values.value))) values.value = valuesFor(next);
  });
  watch(options.hourCycle, () => {
    values.value = valuesFor(value.value);
  });

  function segmentText(
    type: EditableSegmentType,
    segmentValue: number | null,
    active: boolean,
  ): string {
    if (segmentValue === null) {
      if (active && buffer.value.length > 0) {
        return numberFormatters.value(buffer.value.length).format(Number(buffer.value));
      }
      return options.placeholders()?.[type] ?? defaultPlaceholders[type];
    }
    if (type === "dayPeriod") return options.dayPeriodLabels.value[segmentValue === 1 ? 1 : 0];
    return numberFormatters.value(options.padding(type)).format(segmentValue);
  }

  function valueText(type: EditableSegmentType, segmentValue: number | null): string {
    if (segmentValue === null) return options.emptyText() ?? "Empty";
    if (type === "dayPeriod") return options.dayPeriodLabels.value[segmentValue === 1 ? 1 : 0];
    if (type === "month") return `${segmentValue} – ${options.monthName(segmentValue)}`;
    return String(segmentValue);
  }

  const segments = computed<readonly FieldSegmentState[]>(() =>
    options.layout.value.map((part, index): FieldSegmentState => {
      if (part.type === "literal") {
        return {
          type: "literal",
          index,
          text: part.literal,
          value: null,
          min: 0,
          max: 0,
          placeholder: false,
          editable: false,
          label: "",
          valueText: "",
        };
      }
      const segmentValue = values.value[part.type];
      const bounds = segmentBounds(part.type, values.value, options.hourCycle.value);
      return {
        type: part.type,
        index,
        text: segmentText(part.type, segmentValue, activeSegment.value === part.type),
        value: segmentValue,
        min: bounds.min,
        max: bounds.max,
        placeholder: segmentValue === null,
        editable: true,
        label: options.names.value[part.type],
        valueText: valueText(part.type, segmentValue),
      };
    }),
  );
  const filledCount = computed(
    () => editableTypes.value.filter((type) => values.value[type] !== null).length,
  );
  const invalid = computed(() => value.value !== null && options.isInvalid(value.value));
  const fieldState = computed<SegmentedFieldState>(() => {
    if (invalid.value) return "invalid";
    if (filledCount.value === 0) return "empty";
    return filledCount.value === editableTypes.value.length ? "complete" : "partial";
  });

  function commit(next: Value | null, event: Event | null): boolean {
    const previous = value.value;
    const changed = state.set(next);
    if (changed) options.onChange(next, previous, event);
    if (state.controlled.value) {
      void nextTick(() => {
        if (!options.equals(state.value.value, next)) values.value = valuesFor(state.value.value);
      });
    }
    return changed;
  }

  function clampedValues(next: FieldSegmentValues): FieldSegmentValues {
    const day = next.day;
    if (day === null) return next;
    const bounds = segmentBounds("day", next, options.hourCycle.value);
    return day > bounds.max ? { ...next, day: bounds.max } : next;
  }

  function updateSegment(
    type: EditableSegmentType,
    segmentValue: number | null,
    event: Event | null,
  ): void {
    const next = clampedValues({ ...values.value, [type]: segmentValue });
    values.value = next;
    const candidate = options.fromValues(next);
    if (candidate !== null) commit(candidate, event);
    else if (value.value !== null) commit(null, event);
  }

  function segmentElements(): HTMLElement[] {
    const root = options.root.value;
    return root ? [...root.querySelectorAll<HTMLElement>("[role='spinbutton']")] : [];
  }

  function focusSegment(type?: EditableSegmentType, focusOptions?: FocusOptions): boolean {
    const elements = segmentElements();
    const target =
      type === undefined
        ? (elements.find((element) => element.getAttribute("data-placeholder") === "true") ??
          elements[0])
        : elements.find((element) => element.getAttribute("data-segment") === type);
    if (!target || options.disabled()) return false;
    target.focus(focusOptions);
    return true;
  }

  function moveFocus(from: EditableSegmentType, step: -1 | 1): boolean {
    const order = editableTypes.value;
    const next = order[order.indexOf(from) + step];
    return next === undefined ? false : focusSegment(next);
  }

  function step(type: EditableSegmentType, delta: number, event: Event): void {
    const bounds = segmentBounds(type, values.value, options.hourCycle.value);
    const start = options.startValues()[type] ?? bounds.min;
    buffer.value = "";
    updateSegment(type, stepSegmentValue(values.value[type], delta, bounds, start), event);
  }

  function typeDigit(type: EditableSegmentType, digit: string, event: Event): void {
    const bounds = segmentBounds(type, values.value, options.hourCycle.value);
    const result = typeSegmentDigit(buffer.value, digit, bounds, segmentMaxDigits(type));
    buffer.value = result.buffer;
    updateSegment(type, result.value, event);
    if (result.advance) {
      buffer.value = "";
      moveFocus(type, 1);
    }
  }

  function typeDayPeriod(character: string, event: Event): boolean {
    const lower = character.toLocaleLowerCase(options.locale.value);
    const [am, pm] = options.dayPeriodLabels.value;
    const period =
      lower === "a" || am.toLocaleLowerCase(options.locale.value).startsWith(lower)
        ? 0
        : lower === "p" || pm.toLocaleLowerCase(options.locale.value).startsWith(lower)
          ? 1
          : null;
    if (period === null) return false;
    updateSegment("dayPeriod", period, event);
    moveFocus("dayPeriod", 1);
    return true;
  }

  function insertText(type: EditableSegmentType, text: string, event: Event): void {
    for (const character of text) {
      if (type === "dayPeriod") typeDayPeriod(character, event);
      else if (/^[0-9]$/u.test(character)) typeDigit(type, character, event);
    }
  }

  function backspace(type: EditableSegmentType, event: Event): void {
    const current = values.value[type];
    buffer.value = "";
    if (current === null) {
      moveFocus(type, -1);
      return;
    }
    const bounds = segmentBounds(type, values.value, options.hourCycle.value);
    updateSegment(
      type,
      type === "dayPeriod" ? null : backspaceSegmentValue(current, bounds),
      event,
    );
  }

  function onSegmentKeydown(type: EditableSegmentType, event: KeyboardEvent): void {
    if (options.disabled() || event.altKey || event.ctrlKey || event.metaKey) return;
    const rtl = options.direction.value === "rtl";
    const editable = !options.readOnly();
    const bounds = segmentBounds(type, values.value, options.hourCycle.value);
    switch (event.key) {
      case "ArrowLeft":
        event.preventDefault();
        moveFocus(type, rtl ? 1 : -1);
        return;
      case "ArrowRight":
        event.preventDefault();
        moveFocus(type, rtl ? -1 : 1);
        return;
      case "ArrowUp":
      case "ArrowDown":
      case "PageUp":
      case "PageDown": {
        event.preventDefault();
        if (!editable) return;
        const size = event.key.startsWith("Page") ? segmentPageStep(type) : 1;
        step(type, event.key === "ArrowUp" || event.key === "PageUp" ? size : -size, event);
        return;
      }
      case "Home":
      case "End":
        event.preventDefault();
        if (!editable) return;
        buffer.value = "";
        updateSegment(type, event.key === "Home" ? bounds.min : bounds.max, event);
        return;
      case "Backspace":
        event.preventDefault();
        if (editable) backspace(type, event);
        return;
      case "Delete":
        event.preventDefault();
        if (!editable) return;
        buffer.value = "";
        updateSegment(type, null, event);
        return;
      case "Enter":
        event.preventDefault();
        return;
      default:
        if (event.key.length !== 1) return;
        event.preventDefault();
        if (editable) insertText(type, event.key, event);
    }
  }

  function onSegmentBeforeInput(type: EditableSegmentType, event: InputEvent): void {
    event.preventDefault();
    if (options.disabled() || options.readOnly()) return;
    if (event.inputType === "insertText" && event.data) insertText(type, event.data, event);
    else if (event.inputType === "deleteContentBackward") backspace(type, event);
    else if (event.inputType === "deleteContentForward") updateSegment(type, null, event);
  }

  function onSegmentFocus(type: EditableSegmentType): void {
    activeSegment.value = type;
    buffer.value = "";
  }

  function onSegmentBlur(): void {
    activeSegment.value = null;
    buffer.value = "";
  }

  function setValue(next: Value | null): boolean {
    const normalized = options.normalize(next);
    values.value = valuesFor(normalized);
    return commit(normalized, null);
  }

  function reset(): boolean {
    const changed = state.reset();
    values.value = valuesFor(value.value);
    return changed;
  }

  watch(
    options.root,
    (element, _previous, onCleanup) => {
      const form = element?.closest("form");
      if (!form) return;
      const onReset = () => {
        if (!state.controlled.value) reset();
      };
      form.addEventListener("reset", onReset);
      onCleanup(() => form.removeEventListener("reset", onReset));
    },
    { flush: "post", immediate: true },
  );

  return {
    value,
    values: computed(() => values.value),
    segments,
    fieldState,
    invalid,
    controlled: state.controlled,
    focusSegment,
    onSegmentKeydown,
    onSegmentBeforeInput,
    onSegmentFocus,
    onSegmentBlur,
    setValue,
    clear: () => setValue(null),
    reset,
  };
}

/** Inline style that keeps the validation input out of layout and pointer hit testing. */
export const fieldValidationInputStyle =
  "position:absolute;inline-size:1px;block-size:1px;margin:-1px;padding:0;border:0;overflow:hidden;clip-path:inset(50%);white-space:nowrap;opacity:0;pointer-events:none";

/** Inputs of {@link useFieldValidationInput}. */
export interface FieldValidationInputOptions {
  /** Rendered validation input, when any. */
  readonly input: Readonly<ShallowRef<HTMLInputElement | null>>;
  /** Whether the committed value violates min/max/availability or `ariaInvalid`. */
  readonly invalid: () => boolean;
  /** Message reported through `setCustomValidity` while invalid. */
  readonly message: () => string;
  /** Focus the first empty (or first) segment when the form reports the field. */
  readonly focus: () => boolean;
}

/**
 * Keep the visually hidden validation input's custom validity in sync and move
 * focus to the segments when native constraint validation reports it, so
 * `required`, `form.checkValidity()`, `:invalid`, and the browser's validation
 * bubble work for segmented fields. DOM access happens only after mount.
 */
export function useFieldValidationInput(options: FieldValidationInputOptions) {
  watch(
    () => [options.input.value, options.invalid(), options.message()] as const,
    ([input, invalid, message]) => {
      input?.setCustomValidity(invalid ? message : "");
    },
    { flush: "post", immediate: true },
  );

  return {
    onInvalid: () => {
      options.focus();
    },
  };
}
