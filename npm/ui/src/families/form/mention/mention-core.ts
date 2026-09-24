/**
 * DOM-free trigger detection and insertion for Mention.
 *
 * Everything here is pure and deterministic so it can run during SSR, in
 * tests, and inside any editor adapter (textarea, input, contenteditable).
 */

/** One trigger character configuration, e.g. `@` for people or `#` for tags. */
export interface MentionTrigger {
  /** Trigger text that starts a mention token. Usually one character. */
  readonly char: string;

  /**
   * Pattern the typed query must match in full, e.g. `/^[\w-]*$/u`.
   *
   * @default undefined
   */
  readonly pattern?: RegExp;

  /**
   * Allow spaces inside the query (for "@Ada Lovelace"). Newlines always end a token.
   *
   * @default false
   */
  readonly allowSpaces?: boolean;

  /**
   * Minimum query length before the trigger activates.
   *
   * @default 0
   */
  readonly minChars?: number;

  /**
   * Maximum query length scanned backwards from the caret.
   *
   * @default 64
   */
  readonly maxChars?: number;
}

/** Active trigger token around the caret. */
export interface MentionMatch {
  /** Trigger that produced this token. */
  readonly trigger: MentionTrigger;

  /** Text typed after the trigger, up to the caret. */
  readonly query: string;

  /** Index of the trigger's first character. */
  readonly start: number;

  /** Caret index; the token ends here. */
  readonly end: number;
}

/** Text edit produced by inserting a mention. */
export interface MentionEdit {
  /** Full text after the edit. */
  readonly text: string;

  /** Caret index after the inserted text. */
  readonly caret: number;

  /** Replaced range start. */
  readonly start: number;

  /** Replaced range end (exclusive), in the original text. */
  readonly end: number;

  /** Inserted text. */
  readonly inserted: string;
}

/** Transform turning a chosen item into the text inserted for its token. */
export type MentionInsertTransform<T> = (item: T, trigger: MentionTrigger, text: string) => string;

/** The default `@` trigger. */
export const defaultMentionTriggers: readonly MentionTrigger[] = Object.freeze([
  Object.freeze({ char: "@" }),
]);

const boundary = /[\s\p{P}\p{S}]/u;
const whitespace = /\s/u;

function isBoundary(text: string, index: number, trigger: MentionTrigger): boolean {
  if (index === 0) return true;
  const previous = text[index - 1] ?? "";
  // A repeated trigger ("@@") is not a boundary for the second one.
  if (previous === trigger.char.at(-1)) return false;
  return boundary.test(previous);
}

function acceptsQuery(query: string, trigger: MentionTrigger): boolean {
  if (query.includes("\n")) return false;
  // "@@" escapes the trigger, so the doubled form never opens a popup.
  if (query.startsWith(trigger.char)) return false;
  if (trigger.allowSpaces !== true && whitespace.test(query)) return false;
  if (trigger.allowSpaces === true && /^\s/u.test(query)) return false;
  if (query.length < (trigger.minChars ?? 0)) return false;
  if (trigger.pattern !== undefined) {
    trigger.pattern.lastIndex = 0;
    const match = trigger.pattern.exec(query);
    if (match === null || match.index !== 0 || match[0].length !== query.length) return false;
  }
  return true;
}

/**
 * Find the trigger token that ends at `caret`, or `null`.
 *
 * A trigger activates at the start of the text or after whitespace,
 * punctuation, or a symbol, so `mail@example` never opens a popup. The
 * query ends at whitespace unless the trigger allows spaces, and always at a
 * newline. The nearest valid trigger before the caret wins.
 */
export function detectMention(
  text: string,
  caret: number,
  triggers: readonly MentionTrigger[] = defaultMentionTriggers,
): MentionMatch | null {
  if (!Number.isInteger(caret) || caret < 0 || caret > text.length || triggers.length === 0) {
    return null;
  }
  const lookback = Math.max(...triggers.map((trigger) => trigger.maxChars ?? 64)) + 8;
  const floor = Math.max(0, caret - lookback);
  const anySpaces = triggers.some((trigger) => trigger.allowSpaces === true);
  for (let index = caret - 1; index >= floor; index--) {
    const character = text[index] ?? "";
    if (character === "\n") return null;
    for (const trigger of triggers) {
      if (trigger.char.length === 0) continue;
      const start = index - trigger.char.length + 1;
      if (start < 0 || text.slice(start, index + 1) !== trigger.char) continue;
      if (!isBoundary(text, start, trigger)) continue;
      const query = text.slice(index + 1, caret);
      if (query.length > (trigger.maxChars ?? 64)) continue;
      if (acceptsQuery(query, trigger)) return Object.freeze({ end: caret, query, start, trigger });
    }
    if (!anySpaces && whitespace.test(character)) return null;
  }
  return null;
}

/** Whether two matches describe the same token. */
export function isSameMention(left: MentionMatch | null, right: MentionMatch | null): boolean {
  if (left === null || right === null) return left === right;
  return (
    left.start === right.start &&
    left.end === right.end &&
    left.query === right.query &&
    left.trigger.char === right.trigger.char
  );
}

/** Default insertion: the trigger, the item text, and one trailing space. */
export function defaultMentionInsert(text: string, trigger: MentionTrigger): string {
  return `${trigger.char}${text} `;
}

/** Replace the matched token with `inserted` and place the caret after it. */
export function applyMentionEdit(text: string, match: MentionMatch, inserted: string): MentionEdit {
  const before = text.slice(0, match.start);
  let after = text.slice(match.end);
  // Avoid a double space when the insertion ends with whitespace and the
  // remaining text already starts with one.
  if (/\s$/u.test(inserted) && /^[ \t]/u.test(after)) after = after.slice(1);
  return Object.freeze({
    caret: before.length + inserted.length,
    end: match.end,
    inserted,
    start: match.start,
    text: `${before}${inserted}${after}`,
  });
}

/** Normalize text for accent- and case-insensitive filtering. */
export function normalizeMentionText(text: string): string {
  return text
    .normalize("NFKD")
    .replace(/\p{M}+/gu, "")
    .toLocaleLowerCase();
}

/** Default filter: the item text contains the query. */
export function containsMentionFilter(text: string, query: string): boolean {
  const needle = normalizeMentionText(query);
  return needle.length === 0 || normalizeMentionText(text).includes(needle);
}
