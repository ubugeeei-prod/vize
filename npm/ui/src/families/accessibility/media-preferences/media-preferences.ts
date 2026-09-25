/** SSR-safe user media preferences (motion, transparency, forced colors, contrast, color scheme). */
export { default as MediaPreferencesProvider } from "./media-preferences-provider.vue";
export { mediaPreferencesContext } from "./media-preferences-context.ts";
export {
  createMediaPreferencesTracker,
  defaultMediaPreferences,
  mediaPreferenceQueries,
  mergeMediaPreferences,
  readMediaPreferences,
  useForcedColors,
  useMediaPreferences,
  useMediaPreferencesTracker,
  usePrefersColorScheme,
  usePrefersContrast,
  usePrefersReducedMotion,
  usePrefersReducedTransparency,
} from "./media-preferences-runtime.ts";
export type {
  ColorSchemePreference,
  ContrastPreference,
  MediaPreferences,
  MediaPreferencesOverrides,
  MediaPreferencesProviderExpose,
  MediaPreferencesSlotState,
  MediaPreferencesTracker,
  MediaPreferencesTrackerOptions,
} from "./media-preferences-types.ts";
