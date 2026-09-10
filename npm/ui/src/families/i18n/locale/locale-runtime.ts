import { computed, getCurrentInstance, toValue } from "vue";
import type { ComputedRef, MaybeRefOrGetter } from "vue";

import { createContext } from "../../foundations/context/context.ts";

/** Resolved writing direction for a subtree. */
export type TextDirection = "ltr" | "rtl";

/** Direction preference, including locale-driven resolution. */
export type DirectionPreference = TextDirection | "auto";

/** Numbering system identifier such as `latn`, `arab`, or `fullwide`. */
export type LocaleNumberingSystem = string;

/** Calendar system identifier such as `gregory`, `japanese`, or `islamic`. */
export type LocaleCalendar = string;

/** IANA time-zone identifier used by locale-aware date-time helpers. */
export type LocaleTimeZone = string;

/** Temporal-compatible disambiguation policy for local time-zone edges. */
export type LocaleTimeZoneDisambiguation = "compatible" | "earlier" | "later" | "reject";

type LocaleNumberingSystemOptions = {
  readonly numberingSystem?: LocaleNumberingSystem | undefined;
};

/** Number formatter options resolved against the active locale. */
export type LocaleNumberFormatterOptions = Intl.NumberFormatOptions & LocaleNumberingSystemOptions;

/** Date-time formatter options resolved against the active locale. */
export type LocaleDateTimeFormatterOptions = Intl.DateTimeFormatOptions &
  LocaleNumberingSystemOptions;

/** List formatter options resolved against the active locale. */
export type LocaleListFormatterOptions = Intl.ListFormatOptions;

/** Relative-time formatter options resolved against the active locale. */
export type LocaleRelativeTimeFormatterOptions = Intl.RelativeTimeFormatOptions &
  LocaleNumberingSystemOptions;

/** Collator options resolved against the active locale. */
export type LocaleCollatorOptions = Intl.CollatorOptions;

/** Display-name kinds supported by `Intl.DisplayNames`. */
export type LocaleDisplayNamesType =
  | "language"
  | "region"
  | "script"
  | "currency"
  | "calendar"
  | "dateTimeField";

/** Display-name formatter options resolved against the active locale. */
export type LocaleDisplayNamesOptions = Omit<Intl.DisplayNamesOptions, "type"> & {
  /** Code kind to localize. */
  readonly type: LocaleDisplayNamesType;
};

/** Static, ref, or getter-backed formatter options. */
export type LocaleFormatterOptionsInput<Options> = MaybeRefOrGetter<Options | undefined>;

/** Static, ref, or getter-backed display-name options. */
export type LocaleDisplayNamesOptionsInput = MaybeRefOrGetter<LocaleDisplayNamesOptions>;

/** Locale, direction, and formatter defaults published by LocaleProvider. */
export interface LocaleValue {
  readonly locale: string;
  readonly direction: TextDirection;
  readonly numberingSystem: LocaleNumberingSystem | undefined;
  readonly calendar: LocaleCalendar | undefined;
  readonly timeZone: LocaleTimeZone;
  readonly timeZoneDisambiguation: LocaleTimeZoneDisambiguation;
}

const fallbackLocaleValue = "en-US";
const fallbackTimeZoneValue = "UTC";
const fallbackTimeZoneDisambiguation = "compatible";
const setupDiagnostic = "VIZE_UI_LOCALE_SETUP";
const defaultSearchCollatorOptions = Object.freeze({
  sensitivity: "base",
  usage: "search",
} satisfies LocaleCollatorOptions);
const timeZoneDisambiguations = [
  "compatible",
  "earlier",
  "later",
  "reject",
] as const satisfies readonly LocaleTimeZoneDisambiguation[];
const supportedIntlValues =
  typeof Intl.supportedValuesOf === "function" ? Intl.supportedValuesOf.bind(Intl) : undefined;

/** Typed locale context for application and component subtrees. */
export const localeContext = createContext<LocaleValue>("Locale");

/** Fallback locale. */
function fallbackLocale(): string {
  if (typeof document === "undefined") return fallbackLocaleValue;
  return resolveLocale(document.documentElement.lang);
}

/** Fallback direction. */
function fallbackDirection(): TextDirection {
  if (typeof document === "undefined") return "ltr";
  return document.documentElement.dir === "rtl" ? "rtl" : "ltr";
}

/** Fallback context. */
function fallbackLocaleContext(): LocaleValue {
  return {
    locale: fallbackLocale(),
    direction: fallbackDirection(),
    numberingSystem: undefined,
    calendar: undefined,
    timeZone: fallbackTimeZoneValue,
    timeZoneDisambiguation: fallbackTimeZoneDisambiguation,
  };
}

/** Trim Intl identifiers. */
function maybeTrimmedIdentifier(value: string | null | undefined): string | undefined {
  const candidate = value?.trim();
  return candidate && candidate.length > 0 ? candidate : undefined;
}

/** Check Intl support. */
function isSupportedIntlValue(key: "calendar" | "numberingSystem", value: string): boolean {
  if (!supportedIntlValues) return true;
  try {
    return supportedIntlValues(key).includes(value);
  } catch {
    return true;
  }
}

/** Resolve a BCP 47 locale, or `en-US`. */
export function resolveLocale(locale: string): string {
  const candidate = locale.trim();
  if (candidate.length === 0) return fallbackLocaleValue;

  try {
    return Intl.getCanonicalLocales(candidate)[0] ?? fallbackLocaleValue;
  } catch {
    return fallbackLocaleValue;
  }
}

/**
 * Resolve `auto` direction from `Intl.Locale` when the engine exposes it.
 */
export function resolveDirection(direction: DirectionPreference, locale: string): TextDirection {
  if (direction === "ltr" || direction === "rtl") return direction;
  try {
    const info = new Intl.Locale(locale) as Intl.Locale & {
      readonly textInfo?: { readonly direction?: string };
    };
    if (info.textInfo?.direction === "rtl") return "rtl";
  } catch {
    // Invalid locale tags fall back to left-to-right.
  }
  return "ltr";
}

/** Resolve a numbering-system identifier, or the locale default. */
export function resolveNumberingSystem(
  numberingSystem: string | null | undefined,
  locale = fallbackLocaleValue,
): LocaleNumberingSystem | undefined {
  const candidate = maybeTrimmedIdentifier(numberingSystem);
  if (!candidate) return undefined;
  try {
    const resolvedLocale = resolveLocale(locale);
    const canonical = new Intl.Locale(resolvedLocale, { numberingSystem: candidate })
      .numberingSystem;
    if (!canonical || !isSupportedIntlValue("numberingSystem", canonical)) return undefined;
    const resolved = new Intl.NumberFormat(resolvedLocale, {
      numberingSystem: canonical,
    }).resolvedOptions().numberingSystem;
    return resolved === canonical ? resolved : undefined;
  } catch {
    return undefined;
  }
}

/** Resolve a calendar-system identifier, or the locale default. */
export function resolveCalendar(
  calendar: string | null | undefined,
  locale = fallbackLocaleValue,
): LocaleCalendar | undefined {
  const candidate = maybeTrimmedIdentifier(calendar);
  if (!candidate) return undefined;
  try {
    const resolvedLocale = resolveLocale(locale);
    const canonical = new Intl.Locale(resolvedLocale, { calendar: candidate }).calendar;
    if (!canonical || !isSupportedIntlValue("calendar", canonical)) return undefined;
    const resolved = new Intl.DateTimeFormat(resolvedLocale, {
      calendar: canonical,
    }).resolvedOptions().calendar;
    return resolved === canonical ? resolved : undefined;
  } catch {
    return undefined;
  }
}

/** Resolve an IANA time-zone identifier, or `UTC`. */
export function resolveTimeZone(timeZone: string | null | undefined): LocaleTimeZone {
  const candidate = maybeTrimmedIdentifier(timeZone);
  if (!candidate) return fallbackTimeZoneValue;
  try {
    return new Intl.DateTimeFormat(fallbackLocaleValue, {
      timeZone: candidate,
    }).resolvedOptions().timeZone;
  } catch {
    return fallbackTimeZoneValue;
  }
}

/** Resolve a Temporal-compatible time-zone disambiguation policy. */
export function resolveTimeZoneDisambiguation(
  disambiguation: LocaleTimeZoneDisambiguation | null | undefined,
): LocaleTimeZoneDisambiguation {
  return disambiguation && timeZoneDisambiguations.includes(disambiguation)
    ? disambiguation
    : fallbackTimeZoneDisambiguation;
}

/** Create a number formatter. */
export function resolveNumberFormatter(
  locale: string,
  options?: LocaleNumberFormatterOptions,
): Intl.NumberFormat {
  return new Intl.NumberFormat(resolveLocale(locale), options);
}

/** Create a date-time formatter. */
export function resolveDateTimeFormatter(
  locale: string,
  options?: LocaleDateTimeFormatterOptions,
): Intl.DateTimeFormat {
  return new Intl.DateTimeFormat(resolveLocale(locale), options);
}

/** Create a list formatter. */
export function resolveListFormatter(
  locale: string,
  options?: LocaleListFormatterOptions,
): Intl.ListFormat {
  return new Intl.ListFormat(resolveLocale(locale), options);
}

/** Create a relative-time formatter. */
export function resolveRelativeTimeFormatter(
  locale: string,
  options?: LocaleRelativeTimeFormatterOptions,
): Intl.RelativeTimeFormat {
  return new Intl.RelativeTimeFormat(
    resolveLocale(locale),
    options as Intl.RelativeTimeFormatOptions | undefined,
  );
}

/** Create a display-name formatter. */
export function resolveDisplayNames(
  locale: string,
  options: LocaleDisplayNamesOptions,
): Intl.DisplayNames {
  return new Intl.DisplayNames(resolveLocale(locale), options);
}

/** Create a collator. */
export function resolveCollator(locale: string, options?: LocaleCollatorOptions): Intl.Collator {
  return new Intl.Collator(resolveLocale(locale), options);
}

/** Create a search collator with stable typeahead defaults. */
export function resolveSearchCollator(
  locale: string,
  options?: LocaleCollatorOptions,
): Intl.Collator {
  return resolveCollator(locale, { ...defaultSearchCollatorOptions, ...options });
}

/** Require setup. */
function requireLocaleSetup(): void {
  if (!getCurrentInstance()) {
    throw new Error(`${setupDiagnostic}: use inside component setup`);
  }
}

/** Merge number defaults. */
function withNumberingSystem<Options extends LocaleNumberingSystemOptions>(
  options: Options | undefined,
  numberingSystem: LocaleNumberingSystem | undefined,
): Options | undefined {
  if (!numberingSystem || options?.numberingSystem !== undefined) return options;
  return { ...options, numberingSystem } as Options;
}

/** Merge date defaults. */
function withDateTimeDefaults(
  options: LocaleDateTimeFormatterOptions | undefined,
  value: LocaleValue,
): LocaleDateTimeFormatterOptions {
  return {
    ...options,
    calendar: options?.calendar ?? value.calendar,
    numberingSystem: options?.numberingSystem ?? value.numberingSystem,
    timeZone: options?.timeZone ?? value.timeZone,
  };
}

/** Read the nearest locale context, or the document/SSR fallback. */
export function useLocaleValue(): ComputedRef<LocaleValue> {
  requireLocaleSetup();
  const provided = localeContext.useOptional();
  return computed(() => provided ?? fallbackLocaleContext());
}

/** Read the nearest locale, or the document/SSR fallback. */
export function useLocale(): ComputedRef<string> {
  const value = useLocaleValue();
  return computed(() => value.value.locale);
}

/** Read the nearest resolved writing direction, or the document/SSR fallback. */
export function useDirection(): ComputedRef<TextDirection> {
  const value = useLocaleValue();
  return computed(() => value.value.direction);
}

/** Read the nearest numbering system, or the locale default. */
export function useNumberingSystem(): ComputedRef<LocaleNumberingSystem | undefined> {
  const value = useLocaleValue();
  return computed(() => value.value.numberingSystem);
}

/** Read the nearest calendar system, or the locale default. */
export function useCalendar(): ComputedRef<LocaleCalendar | undefined> {
  const value = useLocaleValue();
  return computed(() => value.value.calendar);
}

/** Read the nearest time zone, or the deterministic `UTC` fallback. */
export function useTimeZone(): ComputedRef<LocaleTimeZone> {
  const value = useLocaleValue();
  return computed(() => value.value.timeZone);
}

/** Read the nearest time-zone disambiguation policy. */
export function useTimeZoneDisambiguation(): ComputedRef<LocaleTimeZoneDisambiguation> {
  const value = useLocaleValue();
  return computed(() => value.value.timeZoneDisambiguation);
}

/** Read the nearest locale and memoize a number formatter. */
export function useNumberFormatter(
  options?: LocaleFormatterOptionsInput<LocaleNumberFormatterOptions>,
): ComputedRef<Intl.NumberFormat> {
  const value = useLocaleValue();
  return computed(() =>
    resolveNumberFormatter(
      value.value.locale,
      withNumberingSystem(toValue(options), value.value.numberingSystem),
    ),
  );
}

/** Read the nearest locale and memoize a date-time formatter. */
export function useDateTimeFormatter(
  options?: LocaleFormatterOptionsInput<LocaleDateTimeFormatterOptions>,
): ComputedRef<Intl.DateTimeFormat> {
  const value = useLocaleValue();
  return computed(() =>
    resolveDateTimeFormatter(
      value.value.locale,
      withDateTimeDefaults(toValue(options), value.value),
    ),
  );
}

/** Read the nearest locale and memoize a list formatter. */
export function useListFormatter(
  options?: LocaleFormatterOptionsInput<LocaleListFormatterOptions>,
): ComputedRef<Intl.ListFormat> {
  const locale = useLocale();
  return computed(() => resolveListFormatter(locale.value, toValue(options)));
}

/** Read the nearest locale and memoize a relative-time formatter. */
export function useRelativeTimeFormatter(
  options?: LocaleFormatterOptionsInput<LocaleRelativeTimeFormatterOptions>,
): ComputedRef<Intl.RelativeTimeFormat> {
  const value = useLocaleValue();
  return computed(() =>
    resolveRelativeTimeFormatter(
      value.value.locale,
      withNumberingSystem(toValue(options), value.value.numberingSystem),
    ),
  );
}

/** Read the nearest locale and memoize a display-name formatter. */
export function useDisplayNames(
  options: LocaleDisplayNamesOptionsInput,
): ComputedRef<Intl.DisplayNames> {
  const locale = useLocale();
  return computed(() => resolveDisplayNames(locale.value, toValue(options)));
}

/** Read the nearest locale and memoize a collator. */
export function useCollator(
  options?: LocaleFormatterOptionsInput<LocaleCollatorOptions>,
): ComputedRef<Intl.Collator> {
  const locale = useLocale();
  return computed(() => resolveCollator(locale.value, toValue(options)));
}

/** Read the nearest locale and memoize a search collator. */
export function useSearchCollator(
  options?: LocaleFormatterOptionsInput<LocaleCollatorOptions>,
): ComputedRef<Intl.Collator> {
  const locale = useLocale();
  return computed(() => resolveSearchCollator(locale.value, toValue(options)));
}
