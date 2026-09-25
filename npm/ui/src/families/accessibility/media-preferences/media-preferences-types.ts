import type { ComputedRef, MaybeRefOrGetter } from "vue";

/** User contrast preference from `prefers-contrast`. */
export type ContrastPreference = "custom" | "less" | "more" | "no-preference";

/** User color-scheme preference from `prefers-color-scheme`; `no-preference` before detection. */
export type ColorSchemePreference = "dark" | "light" | "no-preference";

/** Snapshot of the user media preferences the UI adapts to. */
export interface MediaPreferences {
  /** `prefers-reduced-motion: reduce`. */
  readonly reducedMotion: boolean;

  /** `prefers-reduced-transparency: reduce`. */
  readonly reducedTransparency: boolean;

  /** `forced-colors: active` (Windows High Contrast and similar modes). */
  readonly forcedColors: boolean;

  /** `prefers-contrast` value. */
  readonly contrast: ContrastPreference;

  /** `prefers-color-scheme` value. */
  readonly colorScheme: ColorSchemePreference;
}

/** Partial preference overrides, for server hints or explicit user settings. */
export type MediaPreferencesOverrides = Partial<MediaPreferences>;

/** Options accepted by {@link createMediaPreferencesTracker}. */
export interface MediaPreferencesTrackerOptions {
  /**
   * Values used before the tracker starts (for example from client hints or a
   * cookie) so server and hydration renders agree. Reactive sources are
   * re-read until detection starts.
   *
   * @default undefined
   */
  readonly initial?: MaybeRefOrGetter<MediaPreferencesOverrides | undefined>;
}

/** Media-query-backed preference tracker. */
export interface MediaPreferencesTracker {
  /** Latest preferences: initial values until started, detected values afterwards. */
  readonly preferences: ComputedRef<MediaPreferences>;

  /** Whether the tracker currently listens to a window. */
  readonly started: ComputedRef<boolean>;

  /** Read the window's media queries and follow their `change` events. */
  readonly start: (view: Window | null) => void;

  /** Stop listening and keep the last detected values. */
  readonly stop: () => void;

  /** Stop listening permanently. */
  readonly dispose: () => void;
}

/** State exposed to the MediaPreferencesProvider slot. */
export type MediaPreferencesSlotState = MediaPreferences;

/** Public instance exposed by MediaPreferencesProvider. */
export interface MediaPreferencesProviderExpose extends MediaPreferences {
  /** Rendered provider element. */
  readonly element: HTMLDivElement | null;
}
