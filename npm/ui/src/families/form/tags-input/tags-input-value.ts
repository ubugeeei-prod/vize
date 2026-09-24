import type {
  TagsInputBy,
  TagsInputInvalidReason,
  TagsInputValidator,
} from "./tags-input-types.ts";

const whitespaceSeparators = /[\n\r\t]/u;

/** Options for {@link evaluateTag}. */
export interface TagsInputEvaluateOptions<T> {
  /** Equality used for duplicate detection. */
  readonly equals: (left: T, right: T) => boolean;

  /** Accept tags equal to an existing tag. */
  readonly allowDuplicates: boolean;

  /** Maximum tag count, or `undefined` for unlimited. */
  readonly max: number | undefined;

  /** Consumer validator, if any. */
  readonly validate: TagsInputValidator<T> | undefined;

  /** Index being replaced by an inline edit; it is excluded from duplicate and max checks. */
  readonly replaceIndex?: number;
}

/** Rejection verdict returned by {@link evaluateTag}. */
export interface TagsInputRejection {
  /** Stable rejection reason. */
  readonly reason: TagsInputInvalidReason;

  /** Validator-provided message, or `null`. */
  readonly message: string | null;
}

/**
 * Split raw text into trimmed, non-empty tag candidates.
 *
 * Every delimiter separates candidates. With `splitWhitespaceLines`, line breaks
 * and tabs also separate candidates, which is how pasted spreadsheet columns and
 * multi-line lists arrive.
 */
export function splitTagText(
  text: string,
  delimiters: readonly string[],
  splitWhitespaceLines = false,
): string[] {
  let segments = [text];
  for (const delimiter of delimiters) {
    if (delimiter.length === 0) continue;
    segments = segments.flatMap((segment) => segment.split(delimiter));
  }
  if (splitWhitespaceLines) {
    segments = segments.flatMap((segment) => segment.split(whitespaceSeparators));
  }
  return segments.map((segment) => segment.trim()).filter((segment) => segment.length > 0);
}

/** Whether text contains any non-empty delimiter, or a line break or tab when requested. */
export function containsTagDelimiter(
  text: string,
  delimiters: readonly string[],
  splitWhitespaceLines = false,
): boolean {
  if (splitWhitespaceLines && whitespaceSeparators.test(text)) return true;
  return delimiters.some((delimiter) => delimiter.length > 0 && text.includes(delimiter));
}

/**
 * Split text at its last delimiter: committed candidates before it and the
 * trailing fragment the user is still typing.
 */
export function splitTrailingTagText(
  text: string,
  delimiters: readonly string[],
): { readonly complete: string; readonly rest: string } {
  let cut = -1;
  let width = 0;
  for (const delimiter of delimiters) {
    if (delimiter.length === 0) continue;
    const index = text.lastIndexOf(delimiter);
    if (index > cut) {
      cut = index;
      width = delimiter.length;
    }
  }
  if (cut < 0) return { complete: "", rest: text };
  return { complete: text.slice(0, cut), rest: text.slice(cut + width) };
}

/** Resolve the duplicate-detection equality for a `by` policy. */
export function resolveTagEquality<T>(
  by: TagsInputBy<T> | undefined,
): (left: T, right: T) => boolean {
  if (by === undefined) return Object.is;
  if (typeof by === "function") return by;
  return (left, right) => readTagKey(left, by) === readTagKey(right, by);
}

function readTagKey(value: unknown, key: string): unknown {
  if (typeof value !== "object" || value === null) return value;
  return (value as Record<string, unknown>)[key];
}

/** Default display text: strings are shown as-is, other values through `String`. */
export function defaultTagText(tag: unknown): string {
  return typeof tag === "string" ? tag : String(tag);
}

/** Validate one candidate against the current tags. Returns `null` when accepted. */
export function evaluateTag<T>(
  tag: T,
  tags: readonly T[],
  options: TagsInputEvaluateOptions<T>,
): TagsInputRejection | null {
  const others =
    options.replaceIndex === undefined
      ? tags
      : tags.filter((_tag, index) => index !== options.replaceIndex);
  if (options.max !== undefined && others.length >= options.max) {
    return { reason: "max", message: null };
  }
  if (!options.allowDuplicates && others.some((existing) => options.equals(existing, tag))) {
    return { reason: "duplicate", message: null };
  }
  const verdict = options.validate?.(tag, others);
  if (verdict === false) return { reason: "invalid", message: null };
  if (typeof verdict === "string") return { reason: "invalid", message: verdict };
  return null;
}

/** Return a frozen copy of `tags` with `tag` appended. */
export function appendTag<T>(tags: readonly T[], tag: T): readonly T[] {
  return Object.freeze([...tags, tag]);
}

/** Return a frozen copy of `tags` without the tag at `index`. */
export function removeTagAt<T>(tags: readonly T[], index: number): readonly T[] {
  return Object.freeze(tags.filter((_tag, candidate) => candidate !== index));
}

/** Return a frozen copy of `tags` with the tag at `index` replaced. */
export function replaceTagAt<T>(tags: readonly T[], index: number, tag: T): readonly T[] {
  return Object.freeze(tags.map((existing, candidate) => (candidate === index ? tag : existing)));
}

/** Shallow, order-sensitive equality for tag arrays. */
export function areTagListsEqual<T>(left: readonly T[], right: readonly T[]): boolean {
  return left.length === right.length && left.every((tag, index) => Object.is(tag, right[index]));
}

/** One native submission entry rendered by the root for a tag. */
export interface TagsInputHiddenEntry {
  /** Stable render key combining position and submitted text. */
  readonly key: string;

  /** Submitted form value. */
  readonly value: string;
}
