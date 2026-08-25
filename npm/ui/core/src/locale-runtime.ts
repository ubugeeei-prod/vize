import { computed, getCurrentInstance } from "vue";
import type { ComputedRef } from "vue";

import { createContext } from "./context.ts";

/** Resolved writing direction for a subtree. */
export type TextDirection = "ltr" | "rtl";

/** Direction preference, including locale-driven resolution. */
export type DirectionPreference = TextDirection | "auto";

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
  const lang = document.documentElement.lang.trim();
  return lang.length > 0 ? lang : fallbackLocaleValue;
}

function fallbackDirection(): TextDirection {
  if (typeof document === "undefined") return "ltr";
  return document.documentElement.dir === "rtl" ? "rtl" : "ltr";
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
