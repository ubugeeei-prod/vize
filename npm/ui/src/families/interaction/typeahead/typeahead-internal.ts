import { toValue } from "vue";

import type { TypeaheadOptions } from "./typeahead-types.ts";
import type { CollectionKey } from "../../foundations/collection/collection.ts";

const optionDiagnostic = "VIZE_UI_TYPEAHEAD_OPTION";
const inputDiagnostic = "VIZE_UI_TYPEAHEAD_INPUT";
const graphemeSegmenter = new Intl.Segmenter(undefined, { granularity: "grapheme" });

export function splitGraphemes(value: string): string[] {
  return [...graphemeSegmenter.segment(value)].map(({ segment }) => segment);
}

export function readBoolean(
  source: TypeaheadOptions<CollectionKey, unknown>["isDisabled"],
  name: string,
): boolean {
  const value = toValue(source);
  if (value === undefined) return false;
  if (typeof value !== "boolean") {
    throw new TypeError(`${optionDiagnostic}: ${name} must resolve to a boolean`);
  }
  return value;
}

export function readTimeout(source: TypeaheadOptions<CollectionKey, unknown>["timeout"]): number {
  const value = toValue(source) ?? 500;
  if (typeof value !== "number" || !Number.isFinite(value) || value < 0) {
    throw new TypeError(`${optionDiagnostic}: timeout must resolve to a finite number >= 0`);
  }
  return value;
}

export function readGrapheme(value: string): string {
  if (typeof value !== "string" || value.length === 0) {
    throw new TypeError(`${inputDiagnostic}: input must be exactly one Unicode grapheme`);
  }
  const segments = splitGraphemes(value);
  if (segments.length !== 1 || segments[0] !== value) {
    throw new TypeError(`${inputDiagnostic}: input must be exactly one Unicode grapheme`);
  }
  return value;
}

export function isCharacterKey(event: KeyboardEvent): boolean {
  if (event.isComposing || event.key === "Dead" || event.key === "Process") return false;
  const altGraph = event.getModifierState?.("AltGraph") === true;
  if (event.metaKey || (event.ctrlKey && !altGraph) || (event.altKey && !altGraph)) return false;
  try {
    readGrapheme(event.key);
    return true;
  } catch {
    return false;
  }
}

export function validateOptions<Key extends CollectionKey, Value>(
  options: TypeaheadOptions<Key, Value>,
): void {
  const registry = options.registry as Partial<typeof options.registry> | null;
  if (!registry || typeof registry.moveActiveByTextValue !== "function" || !registry.activeKey) {
    throw new TypeError(`${optionDiagnostic}: registry must be a CollectionRegistry`);
  }
  if (options.allowSpace !== undefined && typeof options.allowSpace !== "boolean") {
    throw new TypeError(`${optionDiagnostic}: allowSpace must be a boolean`);
  }
  if (options.onMatch !== undefined && typeof options.onMatch !== "function") {
    throw new TypeError(`${optionDiagnostic}: onMatch must be a function`);
  }
  if (options.collator !== undefined && typeof options.collator.compare !== "function") {
    throw new TypeError(`${optionDiagnostic}: collator must be an Intl.Collator`);
  }
  readTimeout(options.timeout);
  readBoolean(options.isDisabled, "isDisabled");
}
