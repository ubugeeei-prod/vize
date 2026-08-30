import type { NativeSelectValue } from "./native-select-types.ts";

/** Return whether two NativeSelect values are equal in selection order. */
export function areNativeSelectValuesEqual(
  left: NativeSelectValue,
  right: NativeSelectValue,
): boolean {
  if (typeof left === "string" || typeof right === "string") return left === right;
  return left.length === right.length && left.every((value, index) => value === right[index]);
}

/** Normalize a public value for the active native select mode. */
export function normalizeNativeSelectValue(
  value: NativeSelectValue | undefined,
  multiple: boolean,
): NativeSelectValue {
  if (multiple) {
    if (value === undefined) return [];
    return typeof value === "string" ? (value === "" ? [] : [value]) : [...value];
  }
  if (value === undefined) return "";
  return typeof value === "string" ? value : (value[0] ?? "");
}

/** Convert a normalized NativeSelect value to a readonly selected-value list. */
export function nativeSelectSelectedValues(value: NativeSelectValue): readonly string[] {
  return typeof value === "string" ? (value === "" ? [] : [value]) : value;
}
