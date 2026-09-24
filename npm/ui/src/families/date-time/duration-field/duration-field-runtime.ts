import { computed, shallowRef, watch } from "vue";
import type { ComputedRef, ShallowRef } from "vue";

import { useControllableState } from "../../foundations/controllable-state/controllable-state.ts";
import {
  deriveDeterministicId,
  useDeterministicId,
} from "../../foundations/id/deterministic-id.ts";
import { resolveLocale, useLocaleValue } from "../../i18n/locale/locale-runtime.ts";
import type { SegmentedFieldState } from "../date-field/field-segment-runtime.ts";
import {
  backspaceSegmentValue,
  stepSegmentValue,
  typeSegmentDigit,
} from "../date-field/field-segments.ts";
import type { FieldSegmentBounds } from "../date-field/field-segments.ts";
import { durationUnits, formatIsoDuration, isSameDuration, normalizeDuration } from "./duration.ts";
import type { DurationUnit, DurationValue } from "./duration.ts";
import type {
  DurationFieldExpose,
  DurationFieldSlotState,
  DurationSegmentState,
  DurationUnitDisplay,
} from "./duration-field-types.ts";

/** Props read by {@link useDurationField}; every key is present and may be undefined. */
export interface DurationFieldRuntimeProps {
  readonly id: string | null | undefined;
  readonly modelValue: DurationValue | null | undefined;
  readonly defaultValue: DurationValue | null | undefined;
  readonly fields: readonly DurationUnit[] | undefined;
  readonly unitDisplay: DurationUnitDisplay | undefined;
  readonly maxValue: number | undefined;
  readonly locale: string | undefined;
  readonly dir: "ltr" | "rtl" | undefined;
  readonly disabled: boolean | undefined;
  readonly readOnly: boolean | undefined;
  readonly required: boolean | undefined;
  readonly placeholder: string | undefined;
  readonly emptyText: string | undefined;
}

type DurationFieldEmit = {
  (event: "update:modelValue", value: DurationValue | null): void;
  (
    event: "change",
    value: DurationValue | null,
    previous: DurationValue | null,
    nativeEvent: Event | null,
  ): void;
};

type DurationFieldSetupExpose = {
  readonly [Key in keyof DurationFieldExpose]: DurationFieldExpose[Key] extends (
    ...args: never[]
  ) => unknown
    ? DurationFieldExpose[Key]
    : Key extends "root"
      ? Readonly<ShallowRef<HTMLDivElement | null>>
      : ComputedRef<DurationFieldExpose[Key]>;
};

type DurationSegmentValues = Readonly<Record<DurationUnit, number | null>>;

const singularUnits: Readonly<Record<DurationUnit, string>> = {
  years: "year",
  months: "month",
  weeks: "week",
  days: "day",
  hours: "hour",
  minutes: "minute",
  seconds: "second",
};
const displayNameKeys: Readonly<Record<DurationUnit, string>> = {
  years: "year",
  months: "month",
  weeks: "weekOfYear",
  days: "day",
  hours: "hour",
  minutes: "minute",
  seconds: "second",
};
const emptyValues: DurationSegmentValues = Object.freeze({
  years: null,
  months: null,
  weeks: null,
  days: null,
  hours: null,
  minutes: null,
  seconds: null,
});
const defaultFields: readonly DurationUnit[] = Object.freeze(["hours", "minutes"]);

/** Canonical, de-duplicated field list. */
export function normalizeDurationFields(
  fields: readonly DurationUnit[] | undefined,
): readonly DurationUnit[] {
  const requested = new Set(fields ?? defaultFields);
  const result = durationUnits.filter((unit) => requested.has(unit));
  return result.length > 0 ? result : defaultFields;
}

/** Bounds of a unit given which larger units are also edited. */
export function durationUnitBounds(
  unit: DurationUnit,
  fields: readonly DurationUnit[],
  maxValue: number,
): FieldSegmentBounds {
  const has = (candidate: DurationUnit) => fields.includes(candidate);
  const larger = fields.slice(0, fields.indexOf(unit));
  const carried =
    (unit === "months" && has("years")) ||
    (unit === "days" && has("weeks")) ||
    (unit === "hours" && (has("days") || has("weeks"))) ||
    ((unit === "minutes" || unit === "seconds") &&
      larger.some(
        (candidate) => candidate !== "years" && candidate !== "months" && candidate !== unit,
      ));
  if (!carried) return { min: 0, max: maxValue };
  switch (unit) {
    case "months":
      return { min: 0, max: 11 };
    case "days":
      return { min: 0, max: 6 };
    case "hours":
      return { min: 0, max: 23 };
    default:
      return { min: 0, max: 59 };
  }
}

/** DurationField state machine. Must run during setup. */
export function useDurationField(
  props: DurationFieldRuntimeProps,
  emit: DurationFieldEmit,
  root: Readonly<ShallowRef<HTMLDivElement | null>>,
) {
  const localeValue = useLocaleValue();
  const id = useDeterministicId({ id: () => props.id, hint: "duration-field" });
  const locale = computed(() => resolveLocale(props.locale ?? localeValue.value.locale));
  const direction = computed<"ltr" | "rtl">(() =>
    props.dir === "rtl" || props.dir === "ltr" ? props.dir : localeValue.value.direction,
  );
  const fields = computed(() => normalizeDurationFields(props.fields));
  const maxValue = computed(() => {
    const value = Math.trunc(props.maxValue ?? 9_999);
    return Number.isFinite(value) && value >= 1 ? Math.min(value, 999_999) : 9_999;
  });
  const disabled = computed(() => props.disabled === true);
  const readOnly = computed(() => props.readOnly === true);
  const required = computed(() => props.required === true);
  const state = useControllableState<DurationValue | null>({
    value: () => (props.modelValue === undefined ? undefined : normalizeDuration(props.modelValue)),
    defaultValue: () => normalizeDuration(props.defaultValue),
    equals: isSameDuration,
    onChange: (value) => emit("update:modelValue", value),
  });
  const value = computed(() => state.value.value);
  const valuesFor = (next: DurationValue | null): DurationSegmentValues => {
    if (!next) return emptyValues;
    const result: Record<DurationUnit, number | null> = { ...emptyValues };
    for (const unit of fields.value) result[unit] = next[unit] ?? 0;
    return result;
  };
  const values = shallowRef<DurationSegmentValues>(valuesFor(value.value));
  const buffer = shallowRef("");
  const active = shallowRef<DurationUnit | null>(null);

  watch(value, (next) => {
    if (!isSameDuration(next, fromValues(values.value))) values.value = valuesFor(next);
  });
  watch(fields, () => {
    values.value = valuesFor(value.value);
  });

  function fromValues(next: DurationSegmentValues): DurationValue | null {
    const result: { [Unit in DurationUnit]?: number } = {};
    for (const unit of fields.value) {
      const amount = next[unit];
      if (amount === null) return null;
      result[unit] = amount;
    }
    return normalizeDuration(result);
  }

  const formatters = computed(() => {
    const cache = new Map<string, Intl.NumberFormat>();
    return (unit: DurationUnit, display: DurationUnitDisplay) => {
      const key = `${unit}:${display}`;
      let formatter = cache.get(key);
      if (!formatter) {
        formatter = new Intl.NumberFormat(locale.value, {
          style: "unit",
          unit: singularUnits[unit],
          unitDisplay: display,
        });
        cache.set(key, formatter);
      }
      return formatter;
    };
  });
  const names = computed(() => {
    const result: Record<DurationUnit, string> = { ...singularUnits };
    try {
      const displayNames = new Intl.DisplayNames(locale.value, { type: "dateTimeField" });
      for (const unit of durationUnits) {
        const name = displayNames.of(displayNameKeys[unit]);
        if (name && unit !== "weeks") result[unit] = name;
      }
    } catch {
      // English unit names remain.
    }
    return result;
  });
  const plainNumbers = computed(() => new Intl.NumberFormat(locale.value, { useGrouping: false }));

  function affixes(unit: DurationUnit, amount: number): readonly [string, string] {
    const parts = formatters.value(unit, props.unitDisplay ?? "short").formatToParts(amount);
    const index = parts.findIndex((part) => part.type === "integer");
    const join = (slice: readonly Intl.NumberFormatPart[]) =>
      slice.map((part) => part.value).join("");
    return index === -1 ? ["", ""] : [join(parts.slice(0, index)), join(parts.slice(index + 1))];
  }

  const segments = computed<readonly DurationSegmentState[]>(() =>
    fields.value.map((unit): DurationSegmentState => {
      const amount = values.value[unit];
      const bounds = durationUnitBounds(unit, fields.value, maxValue.value);
      const [prefix, suffix] = affixes(unit, amount ?? 0);
      const typed = active.value === unit && amount === null && buffer.value.length > 0;
      return {
        unit,
        value: amount,
        min: bounds.min,
        max: bounds.max,
        text:
          amount !== null
            ? plainNumbers.value.format(amount)
            : typed
              ? plainNumbers.value.format(Number(buffer.value))
              : (props.placeholder ?? "––"),
        prefix,
        suffix,
        placeholder: amount === null,
        label: names.value[unit],
        valueText:
          amount === null
            ? (props.emptyText ?? "Empty")
            : formatters.value(unit, "long").format(amount),
      };
    }),
  );
  const fieldState = computed<SegmentedFieldState>(() => {
    const filled = fields.value.filter((unit) => values.value[unit] !== null).length;
    if (filled === 0) return "empty";
    return filled === fields.value.length ? "complete" : "partial";
  });

  function commit(next: DurationValue | null, event: Event | null): boolean {
    const previous = value.value;
    const changed = state.set(next);
    if (changed) emit("change", next, previous, event);
    return changed;
  }

  function update(unit: DurationUnit, amount: number | null, event: Event | null): void {
    const next = { ...values.value, [unit]: amount };
    values.value = next;
    const candidate = fromValues(next);
    if (candidate !== null) commit(candidate, event);
    else if (value.value !== null) commit(null, event);
  }

  function elements(): HTMLElement[] {
    return root.value ? [...root.value.querySelectorAll<HTMLElement>("[role='spinbutton']")] : [];
  }

  function focus(unit?: DurationUnit, options?: FocusOptions): boolean {
    const all = elements();
    const target =
      unit === undefined
        ? (all.find((element) => element.getAttribute("data-placeholder") === "true") ?? all[0])
        : all.find((element) => element.getAttribute("data-unit") === unit);
    if (!target || disabled.value) return false;
    target.focus(options);
    return true;
  }

  function move(unit: DurationUnit, step: -1 | 1): void {
    const next = fields.value[fields.value.indexOf(unit) + step];
    if (next) focus(next);
  }

  function insertText(unit: DurationUnit, text: string, event: Event): void {
    for (const character of text) {
      if (!/^[0-9]$/u.test(character)) continue;
      const bounds = durationUnitBounds(unit, fields.value, maxValue.value);
      const digits = String(bounds.max).length;
      const result = typeSegmentDigit(buffer.value, character, bounds, digits);
      buffer.value = result.buffer;
      update(unit, result.value, event);
      if (result.advance) {
        buffer.value = "";
        move(unit, 1);
      }
    }
  }

  function backspace(unit: DurationUnit, event: Event): void {
    buffer.value = "";
    const amount = values.value[unit];
    if (amount === null) {
      move(unit, -1);
      return;
    }
    update(unit, backspaceSegmentValue(amount, { min: 0, max: amount }), event);
  }

  function onSegmentKeydown(unit: DurationUnit, event: KeyboardEvent): void {
    if (disabled.value || event.altKey || event.ctrlKey || event.metaKey) return;
    const editable = !readOnly.value;
    const rtl = direction.value === "rtl";
    const bounds = durationUnitBounds(unit, fields.value, maxValue.value);
    switch (event.key) {
      case "ArrowLeft":
      case "ArrowRight":
        event.preventDefault();
        move(unit, (event.key === "ArrowRight") !== rtl ? 1 : -1);
        return;
      case "ArrowUp":
      case "ArrowDown":
      case "PageUp":
      case "PageDown": {
        event.preventDefault();
        if (!editable) return;
        const size = event.key.startsWith("Page") ? 10 : 1;
        const delta = event.key === "ArrowUp" || event.key === "PageUp" ? size : -size;
        buffer.value = "";
        update(unit, stepSegmentValue(values.value[unit], delta, bounds, 0), event);
        return;
      }
      case "Home":
      case "End":
        event.preventDefault();
        if (!editable) return;
        buffer.value = "";
        update(unit, event.key === "Home" ? bounds.min : bounds.max, event);
        return;
      case "Backspace":
        event.preventDefault();
        if (editable) backspace(unit, event);
        return;
      case "Delete":
        event.preventDefault();
        if (!editable) return;
        buffer.value = "";
        update(unit, null, event);
        return;
      default:
        if (event.key.length !== 1 && event.key !== "Enter") return;
        event.preventDefault();
        if (editable && event.key !== "Enter") insertText(unit, event.key, event);
    }
  }

  function onSegmentBeforeInput(unit: DurationUnit, event: InputEvent): void {
    event.preventDefault();
    if (disabled.value || readOnly.value) return;
    if (event.inputType === "insertText" && event.data) insertText(unit, event.data, event);
    else if (event.inputType === "deleteContentBackward") backspace(unit, event);
  }

  function onSegmentFocus(unit: DurationUnit): void {
    active.value = unit;
    buffer.value = "";
  }

  function onSegmentBlur(): void {
    active.value = null;
    buffer.value = "";
  }

  function setValue(next: DurationValue | null): boolean {
    const normalized = normalizeDuration(next);
    values.value = valuesFor(normalized);
    return commit(normalized, null);
  }

  watch(
    root,
    (element, _previous, onCleanup) => {
      const form = element?.closest("form");
      if (!form) return;
      const onReset = () => {
        if (state.controlled.value) return;
        state.reset();
        values.value = valuesFor(value.value);
      };
      form.addEventListener("reset", onReset);
      onCleanup(() => form.removeEventListener("reset", onReset));
    },
    { flush: "post", immediate: true },
  );

  const isoValue = computed(() => (value.value ? formatIsoDuration(value.value) : ""));
  const slotState = computed<DurationFieldSlotState>(() => ({
    value: value.value,
    segments: segments.value,
    state: fieldState.value,
    disabled: disabled.value,
    readOnly: readOnly.value,
    required: required.value,
  }));

  const exposed = {
    root,
    value,
    segments,
    state: fieldState,
    disabled,
    readOnly,
    required,
    focus,
    setValue,
    clear: () => setValue(null),
  } satisfies DurationFieldSetupExpose;

  return {
    id,
    direction,
    isoValue,
    slotState,
    exposed,
    focus,
    segmentId: (unit: DurationUnit) => deriveDeterministicId(id.value, unit),
    onSegmentKeydown,
    onSegmentBeforeInput,
    onSegmentFocus,
    onSegmentBlur,
  };
}
