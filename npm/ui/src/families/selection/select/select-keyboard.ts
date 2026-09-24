/** Keyboard and prop-normalization helpers shared by Select and Combobox. */

const graphemeSegmenter =
  typeof Intl !== "undefined" && typeof Intl.Segmenter === "function"
    ? new Intl.Segmenter(undefined, { granularity: "grapheme" })
    : null;

/** Whether a key event produces exactly one printable grapheme without command modifiers. */
export function isPrintableKey(event: KeyboardEvent): boolean {
  if (event.ctrlKey || event.metaKey || event.altKey || event.isComposing) return false;
  const { key } = event;
  if (key.length === 0 || key === " ") return false;
  if (key.length === 1) return true;
  if (graphemeSegmenter === null) return false;
  let count = 0;
  for (const _segment of graphemeSegmenter.segment(key)) {
    count += 1;
    if (count > 1) return false;
  }
  return count === 1 && !/^[A-Z][a-zA-Z0-9]+$/u.test(key);
}

/**
 * Read a boolean prop whose declared type is a generic parameter.
 *
 * Vue cannot infer a runtime `Boolean` type for `Multiple extends boolean`, so
 * the attribute form `<SelectRoot multiple>` arrives as an empty string.
 */
export function readBooleanProp(value: unknown): boolean {
  return value === true || value === "";
}
