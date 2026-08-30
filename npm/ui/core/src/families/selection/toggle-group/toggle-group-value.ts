import type { ToggleGroupType, ToggleGroupValue } from "./toggle-group-types.ts";

function uniqueValues(values: readonly string[]): readonly string[] {
  return [...new Set(values)];
}

/** Return the empty model value for the requested selection mode. */
export function emptyToggleGroupValue(type: ToggleGroupType): ToggleGroupValue {
  return type === "multiple" ? [] : null;
}

/** Normalize public input into the model shape owned by the active selection mode. */
export function normalizeToggleGroupValue(
  value: ToggleGroupValue | undefined,
  type: ToggleGroupType,
): ToggleGroupValue {
  if (value === undefined || value === null) return emptyToggleGroupValue(type);
  if (type === "single") return Array.isArray(value) ? (value[0] ?? null) : value;
  return uniqueValues(Array.isArray(value) ? value : [value]);
}

/** Return selected values as an immutable array for item membership checks. */
export function getToggleGroupPressedValues(
  value: ToggleGroupValue,
  type: ToggleGroupType,
): readonly string[] {
  const normalized = normalizeToggleGroupValue(value, type);
  if (normalized === null) return [];
  return typeof normalized === "string" ? [normalized] : normalized;
}

/** Test whether a normalized group value currently includes one item value. */
export function hasToggleGroupValue(
  value: ToggleGroupValue,
  type: ToggleGroupType,
  itemValue: string,
): boolean {
  return getToggleGroupPressedValues(value, type).includes(itemValue);
}

/** Compare group values after applying mode-specific normalization. */
export function toggleGroupValueEquals(
  left: ToggleGroupValue,
  right: ToggleGroupValue,
  type: ToggleGroupType,
): boolean {
  const leftValues = getToggleGroupPressedValues(left, type);
  const rightValues = getToggleGroupPressedValues(right, type);
  return (
    leftValues.length === rightValues.length &&
    leftValues.every((value, index) => value === rightValues[index])
  );
}

/** Resolve the next value after one item is requested pressed or unpressed. */
export function getNextToggleGroupValue(
  current: ToggleGroupValue,
  type: ToggleGroupType,
  itemValue: string,
  pressed = !hasToggleGroupValue(current, type, itemValue),
): ToggleGroupValue {
  if (type === "single") {
    if (pressed) return itemValue;
    return hasToggleGroupValue(current, type, itemValue) ? null : current;
  }

  const values = getToggleGroupPressedValues(current, type);
  if (pressed) return values.includes(itemValue) ? values : [...values, itemValue];
  return values.filter((value) => value !== itemValue);
}
