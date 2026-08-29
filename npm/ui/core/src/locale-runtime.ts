import { computed, getCurrentInstance, toValue } from "vue";
import type { ComputedRef, MaybeRefOrGetter } from "vue";

import { createContext } from "./context.ts";

/** Resolved writing direction for a subtree. */
export type TextDirection = "ltr" | "rtl";

/** Direction preference, including locale-driven resolution. */
export type DirectionPreference = TextDirection | "auto";

/** Number formatter options resolved against the active locale. */
export type LocaleNumberFormatterOptions = Intl.NumberFormatOptions;

/** Date-time formatter options resolved against the active locale. */
export type LocaleDateTimeFormatterOptions = Intl.DateTimeFormatOptions;

/** List formatter options resolved against the active locale. */
export type LocaleListFormatterOptions = Intl.ListFormatOptions;

/** Relative-time formatter options resolved against the active locale. */
export type LocaleRelativeTimeFormatterOptions = Intl.RelativeTimeFormatOptions;

/** Static, ref, or getter-backed formatter options. */
export type LocaleFormatterOptionsInput<Options> = MaybeRefOrGetter<Options | undefined>;

/** Locale and direction published by LocaleProvider. */
export interface LocaleValue {
  readonly locale: string;
  readonly direction: TextDirection;
}

const fallbackLocaleValue = "en-US";
const setupDiagnostic = "VIZE_UI_LOCALE_SETUP";

/** Typed locale context for application and component subtrees. */
export const localeContext = createContext<LocaleValue>("Locale");

function fallbackLocale(): string {
  if (typeof document === "undefined") return fallbackLocaleValue;
  return resolveLocale(document.documentElement.lang);
}

function fallbackDirection(): TextDirection {
  if (typeof document === "undefined") return "ltr";
  return document.documentElement.dir === "rtl" ? "rtl" : "ltr";
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
  return new Intl.RelativeTimeFormat(resolveLocale(locale), options);
}

/** Read the nearest locale, or the document/SSR fallback. */
export function useLocale(): ComputedRef<string> {
  if (!getCurrentInstance()) {
    throw new Error(`${setupDiagnostic}: use inside component setup`);
  }
  const provided = localeContext.useOptional();
  return computed(() => provided?.locale ?? fallbackLocale());
}

/** Read the nearest resolved writing direction, or the document/SSR fallback. */
export function useDirection(): ComputedRef<TextDirection> {
  if (!getCurrentInstance()) {
    throw new Error(`${setupDiagnostic}: use inside component setup`);
  }
  const provided = localeContext.useOptional();
  return computed(() => provided?.direction ?? fallbackDirection());
}

/** Read the nearest locale and memoize a number formatter. */
export function useNumberFormatter(
  options?: LocaleFormatterOptionsInput<LocaleNumberFormatterOptions>,
): ComputedRef<Intl.NumberFormat> {
  const locale = useLocale();
  return computed(() => resolveNumberFormatter(locale.value, toValue(options)));
}

/** Read the nearest locale and memoize a date-time formatter. */
export function useDateTimeFormatter(
  options?: LocaleFormatterOptionsInput<LocaleDateTimeFormatterOptions>,
): ComputedRef<Intl.DateTimeFormat> {
  const locale = useLocale();
  return computed(() => resolveDateTimeFormatter(locale.value, toValue(options)));
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
  const locale = useLocale();
  return computed(() => resolveRelativeTimeFormatter(locale.value, toValue(options)));
}
