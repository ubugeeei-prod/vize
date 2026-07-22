import { computed, toValue } from "vue";
import type { ComputedRef, MaybeRefOrGetter } from "vue";

/** Text flow reported by the internationalization runtime. */
export type TextDirection = "ltr" | "rtl";

/** Locale selection options for {@link useLocale}. */
export interface UseLocaleOptions {
  /**
   * Locale detector used when the reactive source has no value.
   *
   * @default navigator.language when available; otherwise undefined
   */
  readonly detect?: () => Intl.Locale | string | null | undefined;

  /**
   * Locale used when neither the source nor detector provides one.
   *
   * @default "en"
   */
  readonly fallback?: Intl.Locale | string;
}

/**
 * Reactive locale metadata and cached formatter factories.
 *
 * Formatter factories propagate `TypeError` and `RangeError` from the
 * platform `Intl` constructors when the supplied options are invalid.
 */
export interface LocaleControls {
  /** Canonical Unicode locale identifier. */
  readonly locale: ComputedRef<string>;

  /** Parsed locale details supplied by the internationalization runtime. */
  readonly details: ComputedRef<Intl.Locale>;

  /** Native writing direction for the active locale. */
  readonly direction: ComputedRef<TextDirection>;

  /** Return a cached number formatter for the active locale and options. */
  readonly number: (options?: Intl.NumberFormatOptions) => Intl.NumberFormat;

  /** Return a cached date and time formatter for the active locale and options. */
  readonly dateTime: (options?: Intl.DateTimeFormatOptions) => Intl.DateTimeFormat;

  /** Return a cached list formatter for the active locale and options. */
  readonly list: (options?: Intl.ListFormatOptions) => Intl.ListFormat;

  /** Return a cached relative-time formatter for the active locale and options. */
  readonly relativeTime: (options?: Intl.RelativeTimeFormatOptions) => Intl.RelativeTimeFormat;
}

const FORMATTER_CACHE_LIMIT = 32;

/**
 * Create reactive locale metadata and platform-native formatter factories.
 *
 * Equivalent formatter options reuse instances. The bounded cache follows the
 * active locale automatically and prevents repeated constructor overhead in
 * reactive render paths.
 *
 * Detection is lazy and guarded: the default detector reads
 * `navigator.language` behind a `typeof` check at call time, so importing and
 * calling this during server rendering is safe, and runtimes without a
 * `navigator` resolve to the fallback locale. The composable owns no timers
 * or listeners, so no scope cleanup is required.
 *
 * @param source Reactive locale source. Empty values defer to the detector.
 * @param options Detection and fallback behavior.
 * @default options {}
 * @throws `RangeError` on first read of the reactive values when the winning
 * candidate is not a structurally valid locale identifier.
 * @returns Reactive locale metadata and cached formatter factories.
 */
export function useLocale(
  source?: MaybeRefOrGetter<Intl.Locale | string | null | undefined>,
  options: UseLocaleOptions = {},
): LocaleControls {
  const locale = computed(() => {
    const candidate =
      (source === undefined ? undefined : toValue(source)) ??
      (options.detect ?? detectBrowserLocale)() ??
      options.fallback ??
      "en";
    return candidate instanceof Intl.Locale
      ? candidate.toString()
      : new Intl.Locale(candidate).toString();
  });
  const details = computed(() => new Intl.Locale(locale.value));
  const direction = computed<TextDirection>(() => details.value.getTextInfo().direction);

  const number = createFormatterCache<Intl.NumberFormatOptions, Intl.NumberFormat>(
    (activeLocale, formatOptions) => new Intl.NumberFormat(activeLocale, formatOptions),
  );
  const dateTime = createFormatterCache<Intl.DateTimeFormatOptions, Intl.DateTimeFormat>(
    (activeLocale, formatOptions) => new Intl.DateTimeFormat(activeLocale, formatOptions),
  );
  const list = createFormatterCache<Intl.ListFormatOptions, Intl.ListFormat>(
    (activeLocale, formatOptions) => new Intl.ListFormat(activeLocale, formatOptions),
  );
  const relativeTime = createFormatterCache<
    Intl.RelativeTimeFormatOptions,
    Intl.RelativeTimeFormat
  >((activeLocale, formatOptions) => new Intl.RelativeTimeFormat(activeLocale, formatOptions));

  return {
    locale,
    details,
    direction,
    number: (formatOptions) => number(locale.value, formatOptions),
    dateTime: (formatOptions) => dateTime(locale.value, formatOptions),
    list: (formatOptions) => list(locale.value, formatOptions),
    relativeTime: (formatOptions) => relativeTime(locale.value, formatOptions),
  };
}

function detectBrowserLocale(): string | undefined {
  return typeof navigator === "undefined" ? undefined : navigator.language;
}

function createFormatterCache<Options extends object, Formatter>(
  create: (locale: string, options?: Options) => Formatter,
): (locale: string, options?: Options) => Formatter {
  const cache = new Map<string, Formatter>();
  return (locale, options) => {
    const key = `${locale}\u0000${serializeOptions(options)}`;
    const cached = cache.get(key);
    if (cached !== undefined) {
      cache.delete(key);
      cache.set(key, cached);
      return cached;
    }
    const formatter = create(locale, options);
    if (cache.size >= FORMATTER_CACHE_LIMIT) {
      const oldest = cache.keys().next().value;
      if (oldest !== undefined) cache.delete(oldest);
    }
    cache.set(key, formatter);
    return formatter;
  };
}

function serializeOptions(options: object | undefined): string {
  if (options === undefined) return "";
  return JSON.stringify(
    Object.entries(options).sort(([left], [right]) => left.localeCompare(right)),
  );
}
