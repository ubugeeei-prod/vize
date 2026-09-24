import { computed, toValue } from "vue";
import type { ComputedRef, MaybeRefOrGetter } from "vue";

import { useLocale } from "./locale.ts";

/** Segmentation units supported by `Intl.Segmenter`. */
export type SegmenterGranularity = "grapheme" | "word" | "sentence";

/** One segment of the text. */
export interface TextSegment {
  /** Segment text. */
  readonly segment: string;

  /** UTF-16 code-unit offset of the segment in the source text. */
  readonly index: number;

  /** Word granularity: whether the segment is a word (not spaces or punctuation). */
  readonly isWordLike: boolean;
}

/** Options for {@link useSegmenter}. */
export interface UseSegmenterOptions {
  /**
   * Segmentation unit exposed through `segments` and `count`.
   *
   * @default "grapheme"
   */
  readonly granularity?: SegmenterGranularity;

  /**
   * Locale whose segmentation rules are used (word boundaries in Japanese or
   * Thai depend on it). Pass it explicitly for hydration-stable output.
   *
   * @default the browser language, otherwise "en"
   */
  readonly locale?: string | Intl.Locale;
}

/** Reactive segmentation returned by {@link useSegmenter}. */
export interface SegmenterControls {
  /** Canonical locale in use. */
  readonly locale: ComputedRef<string>;

  /** Segments at the configured granularity. */
  readonly segments: ComputedRef<readonly TextSegment[]>;

  /** Number of segments at the configured granularity. */
  readonly count: ComputedRef<number>;

  /** User-perceived characters (grapheme clusters), independent of granularity. */
  readonly graphemeCount: ComputedRef<number>;

  /** Word-like segments, independent of granularity. */
  readonly wordCount: ComputedRef<number>;

  /**
   * Truncate the text to at most `maxGraphemes` user-perceived characters,
   * never splitting an emoji or combining sequence.
   *
   * @param maxGraphemes Maximum grapheme clusters kept, including the ellipsis.
   * @param ellipsis Appended when the text was shortened.
   * @default ellipsis "…"
   * @returns The possibly truncated text.
   * @throws {RangeError} `[VIZE_COMPOSE_SEGMENTER_INVALID_LENGTH]` for a
   * negative or non-integer length.
   */
  readonly truncate: (maxGraphemes: number, ellipsis?: string) => string;
}

function browserLanguage(): string | undefined {
  return typeof window === "undefined" ? undefined : window.navigator.language;
}

function toSegments(segmenter: Intl.Segmenter, text: string): readonly TextSegment[] {
  return Array.from(segmenter.segment(text), (data) => ({
    segment: data.segment,
    index: data.index,
    isWordLike: data.isWordLike ?? false,
  }));
}

/**
 * Split text into graphemes, words, or sentences with `Intl.Segmenter`.
 *
 * Counts reflect what users perceive: `"👨‍👩‍👧"` is one grapheme, and
 * word counts skip spaces and punctuation while handling languages without
 * spaces (Japanese, Thai) through the locale. `truncate` shortens text
 * without splitting clusters. Derived state only; deterministic during
 * server rendering when the locale is explicit.
 *
 * @example
 * ```ts
 * const { graphemeCount, truncate } = useSegmenter(bio, { locale: "en" });
 * const remaining = computed(() => 160 - graphemeCount.value);
 * ```
 *
 * @param text Reactive text.
 * @param options Reactive granularity and locale.
 * @default options {}
 * @returns Reactive segments, counts, and truncation.
 */
export function useSegmenter(
  text: MaybeRefOrGetter<string>,
  options: MaybeRefOrGetter<UseSegmenterOptions> = {},
): SegmenterControls {
  const localeControls = useLocale(() => toValue(options).locale, { detect: browserLanguage });
  const segmenterFor = (granularity: SegmenterGranularity) =>
    computed(() => new Intl.Segmenter(localeControls.locale.value, { granularity }));
  const graphemes = segmenterFor("grapheme");
  const words = segmenterFor("word");
  const sentences = segmenterFor("sentence");
  const graphemeSegments = computed(() => toSegments(graphemes.value, toValue(text)));
  const wordSegments = computed(() => toSegments(words.value, toValue(text)));
  const segments = computed(() => {
    const granularity = toValue(options).granularity ?? "grapheme";
    if (granularity === "grapheme") return graphemeSegments.value;
    if (granularity === "word") return wordSegments.value;
    return toSegments(sentences.value, toValue(text));
  });

  const truncate = (maxGraphemes: number, ellipsis = "…"): string => {
    if (!Number.isSafeInteger(maxGraphemes) || maxGraphemes < 0) {
      throw new RangeError(
        `[VIZE_COMPOSE_SEGMENTER_INVALID_LENGTH] maxGraphemes must be a non-negative integer; received ${String(maxGraphemes)}`,
      );
    }
    const all = graphemeSegments.value;
    if (all.length <= maxGraphemes) return toValue(text);
    const ellipsisLength = toSegments(graphemes.value, ellipsis).length;
    const keep = Math.max(0, maxGraphemes - ellipsisLength);
    return (
      all
        .slice(0, keep)
        .map((segment) => segment.segment)
        .join("") + (maxGraphemes >= ellipsisLength ? ellipsis : "")
    );
  };

  return {
    locale: localeControls.locale,
    segments,
    count: computed(() => segments.value.length),
    graphemeCount: computed(() => graphemeSegments.value.length),
    wordCount: computed(() => wordSegments.value.filter((segment) => segment.isWordLike).length),
    truncate,
  };
}
