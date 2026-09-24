import {
  computed,
  getCurrentInstance,
  getCurrentScope,
  onMounted,
  onScopeDispose,
  shallowRef,
  toValue,
} from "vue";
import type { ComputedRef } from "vue";

import { mediaPreferencesContext } from "./media-preferences-context.ts";
import type {
  ColorSchemePreference,
  ContrastPreference,
  MediaPreferences,
  MediaPreferencesOverrides,
  MediaPreferencesTracker,
  MediaPreferencesTrackerOptions,
} from "./media-preferences-types.ts";

const setupDiagnostic = "VIZE_UI_MEDIA_PREFERENCES_SETUP";
const disposedDiagnostic = "VIZE_UI_MEDIA_PREFERENCES_DISPOSED";

/** Media queries observed by the tracker, in evaluation order. */
export const mediaPreferenceQueries = Object.freeze({
  colorSchemeDark: "(prefers-color-scheme: dark)",
  colorSchemeLight: "(prefers-color-scheme: light)",
  contrastCustom: "(prefers-contrast: custom)",
  contrastLess: "(prefers-contrast: less)",
  contrastMore: "(prefers-contrast: more)",
  forcedColors: "(forced-colors: active)",
  reducedMotion: "(prefers-reduced-motion: reduce)",
  reducedTransparency: "(prefers-reduced-transparency: reduce)",
});

/** Preferences assumed before detection: no reduction, no forced colors, no preference. */
export const defaultMediaPreferences: MediaPreferences = Object.freeze({
  colorScheme: "no-preference",
  contrast: "no-preference",
  forcedColors: false,
  reducedMotion: false,
  reducedTransparency: false,
});

/** Merge overrides onto a base snapshot, ignoring `undefined` entries. */
export function mergeMediaPreferences(
  base: MediaPreferences,
  ...overrides: readonly (MediaPreferencesOverrides | undefined)[]
): MediaPreferences {
  let merged: MediaPreferences = base;
  for (const override of overrides) {
    if (!override) continue;
    merged = {
      colorScheme: override.colorScheme ?? merged.colorScheme,
      contrast: override.contrast ?? merged.contrast,
      forcedColors: override.forcedColors ?? merged.forcedColors,
      reducedMotion: override.reducedMotion ?? merged.reducedMotion,
      reducedTransparency: override.reducedTransparency ?? merged.reducedTransparency,
    };
  }
  return Object.freeze(merged);
}

function matches(view: Window, query: string): boolean {
  return typeof view.matchMedia === "function" && view.matchMedia(query).matches;
}

/** Read every preference from a window's media queries synchronously. */
export function readMediaPreferences(view: Window): MediaPreferences {
  let contrast: ContrastPreference = "no-preference";
  if (matches(view, mediaPreferenceQueries.contrastMore)) contrast = "more";
  else if (matches(view, mediaPreferenceQueries.contrastLess)) contrast = "less";
  else if (matches(view, mediaPreferenceQueries.contrastCustom)) contrast = "custom";
  let colorScheme: ColorSchemePreference = "no-preference";
  if (matches(view, mediaPreferenceQueries.colorSchemeDark)) colorScheme = "dark";
  else if (matches(view, mediaPreferenceQueries.colorSchemeLight)) colorScheme = "light";
  return Object.freeze({
    colorScheme,
    contrast,
    forcedColors: matches(view, mediaPreferenceQueries.forcedColors),
    reducedMotion: matches(view, mediaPreferenceQueries.reducedMotion),
    reducedTransparency: matches(view, mediaPreferenceQueries.reducedTransparency),
  });
}

/**
 * Create an SSR-safe preference tracker. Nothing touches `window` until
 * {@link MediaPreferencesTracker.start} runs, so renders before mount use the
 * initial values.
 */
export function createMediaPreferencesTracker(
  options: MediaPreferencesTrackerOptions = {},
): MediaPreferencesTracker {
  const detected = shallowRef<MediaPreferences | null>(null);
  const view = shallowRef<Window | null>(null);
  let release: (() => void) | null = null;
  let disposed = false;

  const stop = (): void => {
    release?.();
    release = null;
    view.value = null;
  };

  const start = (nextView: Window | null): void => {
    if (disposed) throw new Error(`${disposedDiagnostic}: the tracker has been disposed`);
    stop();
    if (nextView === null || typeof nextView.matchMedia !== "function") return;
    const update = (): void => {
      detected.value = readMediaPreferences(nextView);
    };
    const lists = Object.values(mediaPreferenceQueries).map((query) => nextView.matchMedia(query));
    for (const list of lists) list.addEventListener("change", update);
    release = () => {
      for (const list of lists) list.removeEventListener("change", update);
    };
    view.value = nextView;
    update();
  };

  return Object.freeze({
    dispose: () => {
      stop();
      disposed = true;
    },
    preferences: computed(
      () =>
        detected.value ?? mergeMediaPreferences(defaultMediaPreferences, toValue(options.initial)),
    ),
    start,
    started: computed(() => view.value !== null),
    stop,
  });
}

/**
 * Create a tracker bound to the current scope. Inside a component it starts
 * after mount, keeping hydration identical to the server render; in a bare
 * effect scope it starts immediately.
 */
export function useMediaPreferencesTracker(
  options: MediaPreferencesTrackerOptions = {},
): MediaPreferencesTracker {
  if (!getCurrentScope()) {
    throw new Error(`${setupDiagnostic}: use inside component setup or an active effect scope`);
  }
  const tracker = createMediaPreferencesTracker(options);
  const begin = (): void => {
    tracker.start(typeof window === "undefined" ? null : window);
  };
  if (getCurrentInstance()) onMounted(begin);
  else begin();
  onScopeDispose(tracker.dispose);
  return tracker;
}

/** Effective preferences from the nearest provider, or a standalone tracker. */
export function useMediaPreferences(): ComputedRef<MediaPreferences> {
  const provided = mediaPreferencesContext.useOptional();
  if (provided) return provided;
  return useMediaPreferencesTracker().preferences;
}

/** Whether the user asked for reduced motion. */
export function usePrefersReducedMotion(): ComputedRef<boolean> {
  const preferences = useMediaPreferences();
  return computed(() => preferences.value.reducedMotion);
}

/** Whether the user asked for reduced transparency. */
export function usePrefersReducedTransparency(): ComputedRef<boolean> {
  const preferences = useMediaPreferences();
  return computed(() => preferences.value.reducedTransparency);
}

/** Whether a forced-colors (high contrast) mode is active. */
export function useForcedColors(): ComputedRef<boolean> {
  const preferences = useMediaPreferences();
  return computed(() => preferences.value.forcedColors);
}

/** The user's contrast preference. */
export function usePrefersContrast(): ComputedRef<ContrastPreference> {
  const preferences = useMediaPreferences();
  return computed(() => preferences.value.contrast);
}

/** The user's color-scheme preference. */
export function usePrefersColorScheme(): ComputedRef<ColorSchemePreference> {
  const preferences = useMediaPreferences();
  return computed(() => preferences.value.colorScheme);
}
