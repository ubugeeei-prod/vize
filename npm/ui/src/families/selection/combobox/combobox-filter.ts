/**
 * DOM-free text matching for Combobox filtering and inline completion.
 *
 * Matching is case-insensitive and diacritic-insensitive: both sides are
 * NFKD-normalized, stripped of combining marks, and lower-cased with the
 * runtime's default locale rules, so "Ecole" matches "École".
 */

/** Filter deciding whether a value stays visible for the current query. */
export type ComboboxFilter<T> = (value: T, query: string, text: string) => boolean;

const combiningMarks = /\p{M}+/gu;

/** Normalize text for accent- and case-insensitive comparison. */
export function normalizeComboboxText(text: string): string {
  return text.normalize("NFKD").replace(combiningMarks, "").toLocaleLowerCase().trim();
}

/** Default filter: the option text contains the query. */
export function containsComboboxFilter<T>(value: T, query: string, text: string): boolean {
  void value;
  const needle = normalizeComboboxText(query);
  return needle.length === 0 || normalizeComboboxText(text).includes(needle);
}

/** Filter keeping options whose text starts with the query. */
export function startsWithComboboxFilter<T>(value: T, query: string, text: string): boolean {
  void value;
  const needle = normalizeComboboxText(query);
  return needle.length === 0 || normalizeComboboxText(text).startsWith(needle);
}

/**
 * Inline completion for `query` against `text`, or `null` when `text` does
 * not start with it. The completion keeps the user's typed prefix verbatim
 * and appends the option's remaining characters.
 */
export function inlineComboboxCompletion(query: string, text: string): string | null {
  if (query.length === 0 || text.length <= query.length) return null;
  const prefix = text.slice(0, query.length);
  if (normalizeComboboxText(prefix) !== normalizeComboboxText(query)) return null;
  return `${query}${text.slice(query.length)}`;
}
