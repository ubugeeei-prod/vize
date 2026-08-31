/** Match policy for locale-aware text comparisons. */
export type LocaleTextMatchMode = "prefix" | "exact" | "contains";

/** Options for locale-aware normalized text matching. */
export interface LocaleTextMatchOptions {
  /**
   * Locale-aware comparator used after NFC and whitespace normalization.
   *
   * Use a search collator with `sensitivity: "base"` for typeahead and
   * command-palette matching.
   */
  readonly collator: Intl.Collator;

  /**
   * Match the whole value, a leading segment, or any segment.
   *
   * @default "prefix"
   */
  readonly match?: LocaleTextMatchMode;
}

/**
 * Normalize authored, extracted, or searched text without applying a locale.
 *
 * Unicode is normalized to NFC and every whitespace run becomes one ASCII
 * space. Locale-sensitive equality remains the responsibility of a collator.
 */
export function normalizeLocaleText(value: string): string {
  return value.normalize("NFC").trim().replace(/\s+/gu, " ");
}

/** Whether a candidate text starts with a query under locale-aware comparison. */
export function localeTextStartsWith(
  candidate: string,
  query: string,
  collator: Intl.Collator,
): boolean {
  const normalizedCandidate = normalizeLocaleText(candidate);
  const normalizedQuery = normalizeLocaleText(query);
  let codeUnitLength = 0;
  for (const character of normalizedCandidate) {
    codeUnitLength += character.length;
    if (collator.compare(normalizedCandidate.slice(0, codeUnitLength), normalizedQuery) === 0) {
      return true;
    }
  }
  return normalizedQuery.length === 0;
}

/** Whether a candidate text contains a query under locale-aware comparison. */
export function localeTextContains(
  candidate: string,
  query: string,
  collator: Intl.Collator,
): boolean {
  const normalizedCandidate = normalizeLocaleText(candidate);
  const normalizedQuery = normalizeLocaleText(query);
  if (normalizedQuery.length === 0) return true;

  for (const start of codeUnitOffsets(normalizedCandidate)) {
    if (localeTextStartsWith(normalizedCandidate.slice(start), normalizedQuery, collator)) {
      return true;
    }
  }
  return false;
}

/** Match normalized text with a locale-aware comparator. */
export function localeTextMatches(
  candidate: string,
  query: string,
  options: LocaleTextMatchOptions,
): boolean {
  const match = options.match ?? "prefix";
  if (match === "exact") {
    return (
      options.collator.compare(normalizeLocaleText(candidate), normalizeLocaleText(query)) === 0
    );
  }
  if (match === "contains") return localeTextContains(candidate, query, options.collator);
  return localeTextStartsWith(candidate, query, options.collator);
}

function* codeUnitOffsets(value: string): Generator<number> {
  let offset = 0;
  yield offset;
  for (const character of value) {
    offset += character.length;
    if (offset < value.length) yield offset;
  }
}
