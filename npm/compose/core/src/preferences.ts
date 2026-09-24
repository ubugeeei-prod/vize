import { computed } from "vue";
import type { ComputedRef, Ref } from "vue";

import { useMediaQuery } from "./media-query.ts";
import type { UseMediaQueryOptions } from "./media-query.ts";

/** Color scheme preference exposed by {@link usePreferredColorScheme}. */
export type ColorSchemePreference = "dark" | "light" | "no-preference";

/** Contrast preference exposed by {@link usePreferredContrast}. */
export type ContrastPreference = "more" | "less" | "custom" | "no-preference";

/** Options for the multi-valued preference composables. */
export interface UsePreferenceOptions<Preference extends string> extends Omit<
  UseMediaQueryOptions,
  "ssrValue"
> {
  /**
   * Preference exposed during server rendering and whenever no media-query
   * capability exists.
   *
   * @default "no-preference"
   */
  readonly ssrPreference?: Preference;
}

/**
 * Whether the user prefers a dark color scheme.
 *
 * Shares {@link useMediaQuery} semantics: `ssrValue` (default `false`) during
 * server rendering and subscription cleanup with the owning scope.
 *
 * @param options Runtime capability and server fallback.
 * @default options {}
 * @returns Readonly ref for `(prefers-color-scheme: dark)`.
 */
export function usePreferredDark(options: UseMediaQueryOptions = {}): Readonly<Ref<boolean>> {
  return useMediaQuery("(prefers-color-scheme: dark)", options);
}

/**
 * Reactive color scheme preference.
 *
 * @param options Runtime capability and server fallback.
 * @default options {}
 * @returns Computed closed preference.
 */
export function usePreferredColorScheme(
  options: UsePreferenceOptions<ColorSchemePreference> = {},
): ComputedRef<ColorSchemePreference> {
  const fallback = options.ssrPreference ?? "no-preference";
  const query = (value: "dark" | "light"): Readonly<Ref<boolean>> =>
    useMediaQuery(`(prefers-color-scheme: ${value})`, withSsr(options, fallback === value));
  const dark = query("dark");
  const light = query("light");
  return computed(() => (dark.value ? "dark" : light.value ? "light" : "no-preference"));
}

/**
 * Reactive contrast preference.
 *
 * @param options Runtime capability and server fallback.
 * @default options {}
 * @returns Computed closed preference.
 */
export function usePreferredContrast(
  options: UsePreferenceOptions<ContrastPreference> = {},
): ComputedRef<ContrastPreference> {
  const fallback = options.ssrPreference ?? "no-preference";
  const query = (value: "more" | "less" | "custom"): Readonly<Ref<boolean>> =>
    useMediaQuery(`(prefers-contrast: ${value})`, withSsr(options, fallback === value));
  const more = query("more");
  const less = query("less");
  const custom = query("custom");
  return computed(() =>
    more.value ? "more" : less.value ? "less" : custom.value ? "custom" : "no-preference",
  );
}

/**
 * Whether the user prefers reduced transparency.
 *
 * @param options Runtime capability and server fallback.
 * @default options {}
 * @returns Readonly ref for `(prefers-reduced-transparency: reduce)`.
 */
export function usePreferredReducedTransparency(
  options: UseMediaQueryOptions = {},
): Readonly<Ref<boolean>> {
  return useMediaQuery("(prefers-reduced-transparency: reduce)", options);
}

function withSsr<Preference extends string>(
  options: UsePreferenceOptions<Preference>,
  ssrValue: boolean,
): UseMediaQueryOptions {
  return options.host === undefined ? { ssrValue } : { ssrValue, host: options.host };
}
