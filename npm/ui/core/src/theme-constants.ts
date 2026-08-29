import type { ThemeDensityScale, ThemePresetName } from "./theme-types.ts";

/** Cascade layers shipped by the package, in ascending priority order. */
export const themeCascadeLayerOrder = Object.freeze([
  "vize.tokens",
  "vize.ui",
  "vize.preset",
  "vize.policy",
] as const);

/** Attribute whose space-separated values opt a subtree into presets. */
export const themePresetAttribute = "data-vize-theme";

/** Attribute that retunes the density factor for a subtree. */
export const themeDensityAttribute = "data-vize-density";

/** Presets shipped in `@layer vize.preset`, ordered from least to most opinionated. */
export const themePresets: readonly ThemePresetName[] = Object.freeze([
  "headless",
  "atelier",
  "midnight",
  "paper",
  "play",
  "signal",
  "high-contrast",
]);

/** Density factors mirrored from the `data-vize-density` scopes in `theme.css`. */
export const themeDensityScales: Readonly<Record<ThemeDensityScale, string>> = Object.freeze({
  compact: "0.85",
  comfortable: "1.15",
});
