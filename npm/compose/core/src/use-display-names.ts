import { computed, toValue } from "vue";
import type { ComputedRef, MaybeRefOrGetter } from "vue";

import { useLocale } from "./locale.ts";

/** Kinds of codes `Intl.DisplayNames` can name. */
export type DisplayNamesKind =
  | "language"
  | "region"
  | "script"
  | "currency"
  | "calendar"
  | "dateTimeField";

/** Options for {@link useDisplayNames}. */
export interface UseDisplayNamesOptions {
  /** Kind of code to name. */
  readonly type: DisplayNamesKind;

  /**
   * Locale the names are written in. Pass it explicitly for
   * hydration-stable server rendering.
   *
   * @default the browser language, otherwise "en"
   */
  readonly locale?: string | Intl.Locale;

  /**
   * Name length.
   *
   * @default "long"
   */
  readonly style?: "narrow" | "short" | "long";

  /**
   * What to return for a well-formed code without a name: the code itself
   * or `undefined`.
   *
   * @default "code"
   */
  readonly fallback?: "code" | "none";

  /**
   * Language names: `"dialect"` names `"en-GB"` "British English",
   * `"standard"` names it "English (United Kingdom)".
   *
   * @default "dialect"
   */
  readonly languageDisplay?: "dialect" | "standard";
}

/** Reactive display names returned by {@link useDisplayNames}. */
export interface DisplayNamesControls {
  /** Canonical locale in use. */
  readonly locale: ComputedRef<string>;

  /** Display names instance for the current locale and options. */
  readonly displayNames: ComputedRef<Intl.DisplayNames>;

  /** Name of the reactive code; `undefined` for malformed or unnamed codes. */
  readonly name: ComputedRef<string | undefined>;

  /**
   * Name any code. Malformed codes return `undefined` instead of throwing.
   *
   * @param code Language tag, region, script, currency, calendar, or field code.
   * @returns The localized name.
   */
  readonly of: (code: string) => string | undefined;
}

function browserLanguage(): string | undefined {
  return typeof window === "undefined" ? undefined : window.navigator.language;
}

/**
 * Localized names of languages, regions, scripts, currencies, calendars, and
 * date-time fields (`Intl.DisplayNames`): `"JP"` → "Japan", `"ja"` → "日本語"
 * in Japanese, `"EUR"` → "Euro".
 *
 * Malformed codes yield `undefined` rather than the platform `RangeError`,
 * so user-entered codes never crash rendering; invalid *options* still
 * throw on first read. Derived state only: no listeners or timers, and
 * deterministic during server rendering when the locale is explicit.
 *
 * @example
 * ```ts
 * const { name } = useDisplayNames(regionCode, { type: "region", locale: "en" });
 * ```
 *
 * @param code Reactive code to name.
 * @param options Reactive kind, locale, and display options.
 * @returns Reactive name and a naming helper.
 */
export function useDisplayNames(
  code: MaybeRefOrGetter<string | null | undefined>,
  options: MaybeRefOrGetter<UseDisplayNamesOptions>,
): DisplayNamesControls {
  const localeControls = useLocale(() => toValue(options).locale, { detect: browserLanguage });
  const displayNames = computed(() => {
    const { locale: _locale, ...displayOptions } = toValue(options);
    return new Intl.DisplayNames(localeControls.locale.value, displayOptions);
  });
  const of = (value: string): string | undefined => {
    const names = displayNames.value;
    try {
      return names.of(value);
    } catch (error) {
      if (error instanceof RangeError) return undefined;
      throw error;
    }
  };
  return {
    locale: localeControls.locale,
    displayNames,
    name: computed(() => {
      const current = toValue(code);
      return current === null || current === undefined || current === "" ? undefined : of(current);
    }),
    of,
  };
}
