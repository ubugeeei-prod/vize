/**
 * Dirty-field comparison for `useForm`.
 *
 * Equality matches the form's own deep compare (dates by instant, arrays by
 * item, plain objects by own keys). Prototype-bearing values are leaves.
 */

import { isContainer } from "./form-value-paths.ts";

/** Whether two form values are the same, ignoring object identity. */
export function valuesEqual(left: unknown, right: unknown): boolean {
  if (Object.is(left, right)) return true;
  if (left instanceof Date && right instanceof Date) return left.getTime() === right.getTime();
  if (Array.isArray(left) && Array.isArray(right)) {
    return (
      left.length === right.length && left.every((item, index) => valuesEqual(item, right[index]))
    );
  }
  if (isContainer(left) && isContainer(right) && !Array.isArray(left) && !Array.isArray(right)) {
    const keys = new Set([...Object.keys(left), ...Object.keys(right)]);
    for (const key of keys) if (!valuesEqual(left[key], right[key])) return false;
    return true;
  }
  return false;
}

function blankDirty(value: unknown): boolean {
  if (value === undefined) return true;
  if (Array.isArray(value)) return value.every((item) => blankDirty(item));
  if (isContainer(value)) return Object.keys(value).length === 0;
  return false;
}

/**
 * Nested dirty flags of `current` against `previous`.
 *
 * Equal objects become `{}`, equal arrays become `[]`, and equal leaves
 * become `undefined` so callers can omit them.
 */
export function dirtyFieldsOf(current: unknown, previous: unknown): unknown {
  if (valuesEqual(current, previous)) {
    if (Array.isArray(current)) return [];
    if (isContainer(current)) return {};
    return undefined;
  }
  if (Array.isArray(current) || Array.isArray(previous)) {
    const left = Array.isArray(current) ? current : [];
    const right = Array.isArray(previous) ? previous : [];
    const length = Math.max(left.length, right.length);
    return Array.from({ length }, (_, index) => {
      if (index >= left.length || index >= right.length) return true;
      const child = dirtyFieldsOf(left[index], right[index]);
      return blankDirty(child) ? undefined : child;
    });
  }
  if (
    isContainer(current) &&
    isContainer(previous) &&
    !Array.isArray(current) &&
    !Array.isArray(previous)
  ) {
    const result: Record<string, unknown> = {};
    const keys = new Set([...Object.keys(current), ...Object.keys(previous)]);
    for (const key of keys) {
      const child = dirtyFieldsOf(
        Object.hasOwn(current, key) ? current[key] : undefined,
        Object.hasOwn(previous, key) ? previous[key] : undefined,
      );
      if (blankDirty(child)) continue;
      result[key] = child;
    }
    return result;
  }
  return true;
}

function own(record: unknown, key: string): unknown {
  if (!isContainer(record) || Array.isArray(record)) return undefined;
  return Object.hasOwn(record, key) ? record[key] : undefined;
}

function mergeKept(base: unknown, current: unknown, previous: unknown): unknown {
  if (valuesEqual(current, previous)) return base;
  if (Array.isArray(current) && Array.isArray(previous)) {
    return current.map((item, index) => {
      if (index >= previous.length) return item;
      const baseItem = Array.isArray(base) ? base[index] : undefined;
      return mergeKept(baseItem, item, previous[index]);
    });
  }
  if (
    isContainer(current) &&
    isContainer(previous) &&
    !Array.isArray(current) &&
    !Array.isArray(previous)
  ) {
    const baseObject = isContainer(base) && !Array.isArray(base) ? base : {};
    const keys = new Set([
      ...Object.keys(baseObject),
      ...Object.keys(current),
      ...Object.keys(previous),
    ]);
    const result: Record<string, unknown> = {};
    for (const key of keys) {
      const inCurrent = Object.hasOwn(current, key);
      const inPrevious = Object.hasOwn(previous, key);
      if (!inCurrent && inPrevious) continue;
      if (inCurrent && !inPrevious) {
        result[key] = current[key];
        continue;
      }
      if (!inCurrent) {
        if (Object.hasOwn(baseObject, key)) result[key] = baseObject[key];
        continue;
      }
      result[key] = mergeKept(own(baseObject, key), current[key], previous[key]);
    }
    return result;
  }
  return current;
}

/**
 * Reset values that keep user edits.
 *
 * Fields equal to `previous` take `base`. Dirty leaves, added keys, and
 * array-length edits keep `current`. A removed key stays removed.
 */
export function valuesKeepingDirty<Value>(base: Value, current: Value, previous: Value): Value {
  return mergeKept(base, current, previous) as Value;
}
