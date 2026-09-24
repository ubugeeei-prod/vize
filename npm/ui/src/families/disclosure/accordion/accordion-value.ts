import type {
  AccordionModelMap,
  AccordionModelValue,
  AccordionType,
  AccordionValue,
} from "./accordion-types.ts";

const safeSegment = /^[A-Za-z0-9][A-Za-z0-9_-]*$/;

function isValueList<Value extends AccordionValue>(
  value: Value | null | readonly Value[],
): value is readonly Value[] {
  return Array.isArray(value);
}

/** Normalize a single or multiple model value to the list of open items. */
export function accordionOpenValues<Value extends AccordionValue>(
  model: Value | null | readonly Value[] | undefined,
): readonly Value[] {
  if (model === undefined || model === null) return [];
  if (isValueList(model)) return [...new Set(model)];
  return [model];
}

/** Convert open items back to the model shape selected by `type`. */
export function toAccordionModel<Value extends AccordionValue, Type extends AccordionType>(
  type: Type,
  values: readonly Value[],
): AccordionModelValue<Value, Type>;
export function toAccordionModel<Value extends AccordionValue>(
  type: AccordionType,
  values: readonly Value[],
): AccordionModelMap<Value>[AccordionType] {
  const models: AccordionModelMap<Value> = {
    multiple: values,
    single: values[0] ?? null,
  };
  return models[type];
}

/** Compare two open-item lists by membership and order. */
export function accordionValuesEqual<Value extends AccordionValue>(
  left: readonly Value[],
  right: readonly Value[],
): boolean {
  if (left.length !== right.length) return false;
  return left.every((value, index) => Object.is(value, right[index]));
}

/** Create a deterministic, DOM-id-safe segment for an item value. */
export function getAccordionValueIdSegment(value: AccordionValue): string {
  const text = typeof value === "number" ? `n${String(value)}` : value;
  if (safeSegment.test(text)) return `item-${text}`;

  const readable = text
    .replaceAll(/[^A-Za-z0-9_-]+/g, "-")
    .replaceAll(/^-+|-+$/g, "")
    .slice(0, 32);
  return `item-${readable || "empty"}-${hashAccordionValue(text)}`;
}

function hashAccordionValue(value: string): string {
  let hash = 0x811c9dc5;
  for (let index = 0; index < value.length; index++) {
    hash ^= value.charCodeAt(index);
    hash = Math.imul(hash, 0x01000193) >>> 0;
  }
  return hash.toString(36);
}
