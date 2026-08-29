/** Semantic color roles. */
export type ThemeColorToken =
  | "canvas"
  | "surface"
  | "text"
  | "text-muted"
  | "accent"
  | "accent-contrast"
  | "border"
  | "danger";

/** Typography tokens: families, the size scale, line heights, and weights. */
export type ThemeTypeToken =
  | "family-sans"
  | "family-mono"
  | `size-${"xs" | "sm" | "md" | "lg" | "xl" | "2xl"}`
  | `leading-${"tight" | "normal" | "loose"}`
  | `weight-${"regular" | "medium" | "bold"}`;

/** Named steps on the density-responsive space scale. */
export type ThemeSpaceToken = "xs" | "sm" | "md" | "lg" | "xl" | "2xl" | "3xl";

/** Density-responsive control sizes. */
export type ThemeSizeToken = "control-sm" | "control-md" | "control-lg";

/** Corner radii. */
export type ThemeRadiusToken = "sm" | "md" | "lg" | "full";

/** Border widths. */
export type ThemeBorderToken = "width-thin" | "width-thick";

/** Elevation roles rendered as box shadows. */
export type ThemeElevationToken = "raised" | "overlay" | "floating";

/** Opacity roles. */
export type ThemeOpacityToken = "muted" | "disabled";

/** Layered-surface slots on the shared z-index registry. */
export type ThemeZIndexToken = "sticky" | "dropdown" | "overlay" | "toast";

/** Focus-ring geometry and color. */
export type ThemeFocusToken = "ring-width" | "ring-offset" | "ring-color";

/** Independent token groups that can be composed with any preset. */
export type ThemeTokenPackName =
  | "color"
  | "typography"
  | "space"
  | "size"
  | "radius"
  | "border"
  | "elevation"
  | "opacity"
  | "z-index"
  | "focus"
  | "density";

/** Suffix of one `--vize-ui-*` theme custom property. */
export type ThemeTokenName =
  | `color-${ThemeColorToken}`
  | `type-${ThemeTypeToken}`
  | `space-${ThemeSpaceToken}`
  | `size-${ThemeSizeToken}`
  | `radius-${ThemeRadiusToken}`
  | `border-${ThemeBorderToken}`
  | `elevation-${ThemeElevationToken}`
  | `opacity-${ThemeOpacityToken}`
  | `z-${ThemeZIndexToken}`
  | `focus-${ThemeFocusToken}`
  | "density";

/** Custom-property overrides applied by {@link setThemeTokens}. */
export type ThemeTokenOverrides = {
  readonly [Name in ThemeTokenName]?: string;
};

/** Presets accepted in the space-separated `data-vize-theme` attribute. */
export type ThemePresetName =
  | "headless"
  | "atelier"
  | "midnight"
  | "paper"
  | "play"
  | "signal"
  | "high-contrast";

/** Density scopes accepted in the `data-vize-density` attribute. */
export type ThemeDensityScale = "compact" | "comfortable";

/** Attribute names owned by the theme scope helpers. */
export type ThemeScopeAttributeName = "data-vize-theme" | "data-vize-density";

/** Serialized attributes that can be rendered on any Vue root or nested scope. */
export type ThemeScopeAttributes = Partial<Record<ThemeScopeAttributeName, string>>;

/** Options for a declarative theme scope. */
export interface ThemeScopeOptions {
  /** Opt-in preset name or array-composed preset names. */
  readonly presets?: ThemePresetName | readonly ThemePresetName[];

  /**
   * Density scale applied through inherited custom properties.
   *
   * @default undefined
   */
  readonly density?: ThemeDensityScale;
}

/** Storage keys consumed by the no-flash theme bootstrap script. */
export type ThemeScopeStorageKeyName = "presets" | "density";

/** Options for {@link createThemeBootstrapScript}. */
export interface ThemeBootstrapOptions {
  /**
   * Storage keys to read before first paint.
   *
   * @default { presets: "vize-ui-theme", density: "vize-ui-density" }
   */
  readonly storageKeys?: Partial<Record<ThemeScopeStorageKeyName, string>>;

  /**
   * Server-rendered fallback attributes used when storage is empty or blocked.
   *
   * @default undefined
   */
  readonly fallback?: ThemeScopeOptions;
}
