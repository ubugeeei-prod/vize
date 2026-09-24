/**
 * Pure value helpers shared by Select and Combobox.
 *
 * Both families store their selection internally as a readonly array of
 * consumer-owned values and convert it to the public `T | null` or
 * `readonly T[]` model only at the component boundary. Every helper here is
 * DOM-free, SSR-safe, and deterministic.
 */

/**
 * Public model type: `readonly T[]` when `Multiple` is `true`, otherwise `T | null`.
 *
 * `null` represents "nothing selected" in single mode so that `undefined`
 * stays reserved for "uncontrolled".
 */
export type SelectModelValue<T, Multiple extends boolean> = Multiple extends true
  ? readonly T[]
  : T | null;

/** Property keys of object values usable as a comparison key. */
export type SelectValueKey<T> = T extends object ? Extract<keyof T, string> : never;

/** Custom value equality used to match options with the current selection. */
export type SelectValueComparator<T> = (left: T, right: T) => boolean;

/**
 * Value comparison strategy: a property key of object values (compared with
 * `Object.is`) or a custom equality function. Omitted means `Object.is`.
 */
export type SelectBy<T> = SelectValueKey<T> | SelectValueComparator<T>;

/** Resolved equality function for one family instance. */
export type SelectValueEquality<T> = (left: T, right: T) => boolean;

const serializeDiagnostic = "VIZE_UI_SELECT_SERIALIZE";

function readKey(value: unknown, key: string): unknown {
  if (typeof value !== "object" || value === null) return value;
  return Reflect.get(value, key);
}

/** Resolve a `by` strategy into one equality function. */
export function createSelectValueEquality<T>(by: SelectBy<T> | undefined): SelectValueEquality<T> {
  if (by === undefined) return Object.is;
  if (typeof by === "function") return by;
  const key: string = by;
  return (left, right) =>
    Object.is(left, right) || Object.is(readKey(left, key), readKey(right, key));
}

/** Return the index of `value` inside `values`, or `-1`. */
export function indexOfSelectValue<T>(
  values: readonly T[],
  value: T,
  equals: SelectValueEquality<T>,
): number {
  for (let index = 0; index < values.length; index++) {
    const candidate = values[index] as T;
    if (equals(candidate, value)) return index;
  }
  return -1;
}

/** Return whether `values` contains `value` under `equals`. */
export function includesSelectValue<T>(
  values: readonly T[],
  value: T,
  equals: SelectValueEquality<T>,
): boolean {
  return indexOfSelectValue(values, value, equals) >= 0;
}

/** Remove duplicates while preserving first-seen order. */
export function uniqueSelectValues<T>(
  values: readonly T[],
  equals: SelectValueEquality<T>,
): readonly T[] {
  const unique: T[] = [];
  for (const value of values) {
    if (!includesSelectValue(unique, value, equals)) unique.push(value);
  }
  return Object.freeze(unique);
}

function isValueList<T>(value: readonly T[] | T): value is readonly T[] {
  return Array.isArray(value);
}

/**
 * Normalize a public model value into the internal readonly selection list.
 *
 * In multiple mode arrays are de-duplicated and a stray scalar becomes a
 * one-item list. In single mode `null`/`undefined` are empty and any other
 * value (including an array value when `T` itself is an array) is one item.
 */
export function toSelectList<T>(
  value: readonly T[] | T | null | undefined,
  multiple: boolean,
  equals: SelectValueEquality<T>,
): readonly T[] {
  if (value === undefined || value === null) return Object.freeze([]);
  if (multiple) {
    return isValueList(value) ? uniqueSelectValues(value, equals) : Object.freeze([value]);
  }
  return isValueList(value) && value.length === 0 ? Object.freeze([]) : singleList(value);
}

function singleList<T>(value: readonly T[] | T): readonly T[] {
  // A single-mode value is always exactly one option value; when `T` is
  // itself an array type the array is that value, never a list.
  return Object.freeze([value as T]);
}

/**
 * Convert the internal selection list into the public model for `multiple`.
 *
 * The conditional return type cannot be proven from a runtime boolean, so the
 * single narrowing lives here and nowhere else in the family.
 */
export function fromSelectList<T, Multiple extends boolean>(
  values: readonly T[],
  multiple: boolean,
): SelectModelValue<T, Multiple> {
  const model: readonly T[] | T | null = multiple
    ? Object.freeze([...values])
    : (values[0] ?? null);
  return model as SelectModelValue<T, Multiple>;
}

/** Whether two selection lists hold the same values in the same order. */
export function areSelectListsEqual<T>(
  left: readonly T[],
  right: readonly T[],
  equals: SelectValueEquality<T>,
): boolean {
  if (left === right) return true;
  if (left.length !== right.length) return false;
  for (let index = 0; index < left.length; index++) {
    if (!equals(left[index] as T, right[index] as T)) return false;
  }
  return true;
}

/** Select `value` under the current mode: replace in single, append in multiple. */
export function selectInList<T>(
  values: readonly T[],
  value: T,
  multiple: boolean,
  equals: SelectValueEquality<T>,
): readonly T[] {
  if (!multiple) return Object.freeze([value]);
  return includesSelectValue(values, value, equals) ? values : Object.freeze([...values, value]);
}

/** Toggle `value` under the current mode. Single mode always selects. */
export function toggleInList<T>(
  values: readonly T[],
  value: T,
  multiple: boolean,
  equals: SelectValueEquality<T>,
): readonly T[] {
  if (!multiple) return Object.freeze([value]);
  const index = indexOfSelectValue(values, value, equals);
  if (index < 0) return Object.freeze([...values, value]);
  return Object.freeze(values.filter((_, candidate) => candidate !== index));
}

/** Remove `value` from the selection. */
export function removeFromList<T>(
  values: readonly T[],
  value: T,
  equals: SelectValueEquality<T>,
): readonly T[] {
  const index = indexOfSelectValue(values, value, equals);
  if (index < 0) return values;
  return Object.freeze(values.filter((_, candidate) => candidate !== index));
}

/**
 * Serialize one option value for native form submission.
 *
 * Strings pass through; numbers, bigints, and booleans use `String`; objects
 * use the `by` key when one is configured and JSON otherwise.
 */
export function serializeSelectValue<T>(value: T, by: SelectBy<T> | undefined): string {
  if (typeof value === "string") return value;
  if (typeof value === "number" || typeof value === "bigint" || typeof value === "boolean") {
    return String(value);
  }
  if (value === null || value === undefined) return "";
  if (typeof by === "string") return serializeSelectValue(readKey(value, by), undefined);
  if (typeof value === "symbol" || typeof value === "function") {
    throw new TypeError(`${serializeDiagnostic}: provide formValue() for non-serializable values`);
  }
  return JSON.stringify(value) ?? "";
}

/** Default human-readable text for a value when no label source exists. */
export function defaultSelectText<T>(value: T, by: SelectBy<T> | undefined): string {
  if (typeof value === "object" && value !== null) {
    for (const key of ["label", "name", "title", "text"] as const) {
      const candidate = readKey(value, key);
      if (typeof candidate === "string") return candidate;
    }
  }
  return serializeSelectValue(value, by);
}
