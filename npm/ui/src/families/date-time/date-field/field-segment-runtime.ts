import { computed, nextTick, shallowRef, watch } from "vue";
import type { ComputedRef, ShallowRef } from "vue";

import { useControllableState } from "../../foundations/controllable-state/controllable-state.ts";
import {
  backspaceSegmentValue,
  emptySegmentValues,
  segmentBounds,
  segmentMaxDigits,
  segmentPageStep,
  stepSegmentValue,
  typeSegmentDigit,
} from "./field-segments.ts";
import type {
  EditableSegmentType,
  FieldSegmentLayoutPart,
  FieldSegmentType,
  FieldSegmentValues,
  SegmentHourCycle,
} from "./field-segments.ts";

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
