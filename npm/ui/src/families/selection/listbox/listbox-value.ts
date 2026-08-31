import type { ListboxSelectionMode, ListboxValue } from "./listbox-types.ts";

const valueDiagnostic = "VIZE_UI_LISTBOX_VALUE";

/** Normalize consumer-owned values to the active Listbox selection model. */
export function normalizeListboxValue(
  value: ListboxValue | undefined,
  selectionMode: ListboxSelectionMode,
): ListboxValue {
  if (selectionMode === "multiple") return uniqueValues(value);
  if (isListboxMultipleValue(value)) return value[0] ?? null;
  if (value === undefined) return null;
  if (value === null || typeof value === "string") return value;
  throw new TypeError(`${valueDiagnostic}: single selection values must be strings or null`);
}

/** Return selected values as a stable readonly array. */
export function listboxSelectedValues(value: ListboxValue): readonly string[] {
  if (isListboxMultipleValue(value)) return Object.freeze([...value]);
  return value === null ? Object.freeze([]) : Object.freeze([value]);
}

/** Return whether two normalized Listbox values represent the same selection. */
export function areListboxValuesEqual(left: ListboxValue, right: ListboxValue): boolean {
  if (Array.isArray(left) || Array.isArray(right)) {
    const leftValues = listboxSelectedValues(left);
    const rightValues = listboxSelectedValues(right);
    return (
      leftValues.length === rightValues.length &&
      leftValues.every((value, index) => value === rightValues[index])
    );
  }
  return Object.is(left, right);
}

/** Return a value with the option selected under the current model. */
export function selectListboxValue(
  current: ListboxValue,
  value: string,
  selectionMode: ListboxSelectionMode,
): ListboxValue {
  if (selectionMode === "single") return value;
  const values = listboxSelectedValues(current);
  return values.includes(value) ? values : Object.freeze([...values, value]);
}

/** Return a value with the option toggled under the current model. */
export function toggleListboxValue(
  current: ListboxValue,
  value: string,
  selectionMode: ListboxSelectionMode,
): ListboxValue {
  if (selectionMode === "single") return value;
  const values = listboxSelectedValues(current);
  return values.includes(value)
    ? Object.freeze(values.filter((candidate) => candidate !== value))
    : Object.freeze([...values, value]);
}

/** Empty value for the current Listbox selection model. */
export function emptyListboxValue(selectionMode: ListboxSelectionMode): ListboxValue {
  return selectionMode === "multiple" ? Object.freeze([]) : null;
}

function uniqueValues(value: ListboxValue | undefined): readonly string[] {
  const values = isListboxMultipleValue(value)
    ? value
    : value === undefined || value === null
      ? []
      : [value];
  const unique: string[] = [];
  for (const item of values) {
    if (typeof item !== "string") {
      throw new TypeError(`${valueDiagnostic}: multiple selection values must be strings`);
    }
    if (!unique.includes(item)) unique.push(item);
  }
  return Object.freeze(unique);
}

function isListboxMultipleValue(value: ListboxValue | undefined): value is readonly string[] {
  return Array.isArray(value);
}
