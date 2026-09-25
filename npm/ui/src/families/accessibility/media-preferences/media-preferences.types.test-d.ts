/** Compile-only assertions for the public MediaPreferences contract. */

import type { ComputedRef } from "vue";

import type {
  ColorSchemePreference,
  ContrastPreference,
  MediaPreferences,
  MediaPreferencesOverrides,
  MediaPreferencesProviderExpose,
} from "./media-preferences.ts";
import {
  MediaPreferencesProvider,
  useForcedColors,
  useMediaPreferences,
  usePrefersColorScheme,
  usePrefersContrast,
  usePrefersReducedMotion,
  usePrefersReducedTransparency,
} from "./media-preferences.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

declare const exposed: MediaPreferencesProviderExpose;

type _Contrast = Expect<Equal<ContrastPreference, "custom" | "less" | "more" | "no-preference">>;
type _Scheme = Expect<Equal<ColorSchemePreference, "dark" | "light" | "no-preference">>;
type _Overrides = Expect<Equal<MediaPreferencesOverrides, Partial<MediaPreferences>>>;
type _Motion = Expect<Equal<ReturnType<typeof usePrefersReducedMotion>, ComputedRef<boolean>>>;
type _Transparency = Expect<
  Equal<ReturnType<typeof usePrefersReducedTransparency>, ComputedRef<boolean>>
>;
type _Forced = Expect<Equal<ReturnType<typeof useForcedColors>, ComputedRef<boolean>>>;
type _UseContrast = Expect<
  Equal<ReturnType<typeof usePrefersContrast>, ComputedRef<ContrastPreference>>
>;
type _UseScheme = Expect<
  Equal<ReturnType<typeof usePrefersColorScheme>, ComputedRef<ColorSchemePreference>>
>;
type _All = Expect<Equal<ReturnType<typeof useMediaPreferences>, ComputedRef<MediaPreferences>>>;
type _Exposed = Expect<Equal<typeof exposed.contrast, ContrastPreference>>;

const props: InstanceType<typeof MediaPreferencesProvider>["$props"] = {
  force: { reducedMotion: true },
  initial: { colorScheme: "dark", contrast: "more" },
};

// @ts-expect-error contrast is a closed union.
const badContrast: MediaPreferencesOverrides = { contrast: "high" };

const badMotion: InstanceType<typeof MediaPreferencesProvider>["$props"] = {
  // @ts-expect-error reduced motion is boolean.
  force: { reducedMotion: "reduce" },
};

void badContrast;
void badMotion;
void props;
