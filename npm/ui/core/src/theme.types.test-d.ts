/** Compile-only assertions for the public theme contract. */

import {
  setThemeTokens,
  themeCascadeLayerOrder,
  themeDensityScales,
  themeTokenPackNames,
  themeTokens,
  themeTokensForPack,
  themeTokenVar,
} from "./theme.ts";
import type {
  ThemeColorToken,
  ThemeDensityScale,
  ThemeElevationToken,
  ThemePresetName,
  ThemeRadiusToken,
  ThemeSpaceToken,
  ThemeTokenPackName,
  ThemeTokenName,
  ThemeZIndexToken,
} from "./theme.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

export const colors: readonly ThemeColorToken[] = [
  "canvas",
  "surface",
  "text",
  "text-muted",
  "accent",
  "accent-contrast",
  "border",
  "danger",
];
export const space: readonly ThemeSpaceToken[] = ["xs", "sm", "md", "lg", "xl", "2xl", "3xl"];
export const radii: readonly ThemeRadiusToken[] = ["sm", "md", "lg", "full"];
export const elevations: readonly ThemeElevationToken[] = ["raised", "overlay", "floating"];
export const layers: readonly ThemeZIndexToken[] = ["sticky", "dropdown", "overlay", "toast"];

export const reference: string = themeTokenVar("color-accent");
export const typeReference: string = themeTokenVar("type-size-md");
export const densityReference: string = themeTokenVar("density");
export const colorPack: readonly ThemeTokenName[] = themeTokensForPack("color");
export const packName: ThemeTokenPackName = "typography";

type _TokenNamesAreClosed = Expect<
  Equal<
    Extract<ThemeTokenName, "color-canvas" | "space-2xl" | "density">,
    "color-canvas" | "space-2xl" | "density"
  >
>;
type _PresetNamesAreClosed = Expect<
  Equal<
    ThemePresetName,
    "headless" | "atelier" | "midnight" | "paper" | "play" | "signal" | "high-contrast"
  >
>;
type _DensityScalesAreClosed = Expect<Equal<ThemeDensityScale, "compact" | "comfortable">>;
type _TokenPackNamesAreClosed = Expect<
  Equal<
    ThemeTokenPackName,
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
    | "density"
  >
>;
type _LayerOrderIsLiteral = Expect<
  Equal<
    (typeof themeCascadeLayerOrder)[number],
    "vize.tokens" | "vize.ui" | "vize.preset" | "vize.policy"
  >
>;

export const restore: () => void = setThemeTokens(document.createElement("div"), {
  "color-accent": "oklch(0.6 0.2 300)",
  "focus-ring-width": "3px",
});

// @ts-expect-error unknown token names never compile.
themeTokenVar("color-bogus");
// @ts-expect-error the token record is readonly.
themeTokens["color-canvas"] = "red";
// @ts-expect-error the density record is readonly.
themeDensityScales.compact = "0.5";
// @ts-expect-error the token pack names are readonly.
themeTokenPackNames[0] = "focus";
// @ts-expect-error token pack names are closed.
themeTokensForPack("paint");
// @ts-expect-error the layer order is readonly.
themeCascadeLayerOrder[0] = "vize.ui";
// @ts-expect-error overrides only accept known token names.
setThemeTokens(document.createElement("div"), { "space-huge": "4rem" });
// @ts-expect-error preset unions reject arbitrary strings.
export const badPreset: ThemePresetName = "baroque";
// @ts-expect-error density unions reject arbitrary strings.
export const badDensity: ThemeDensityScale = "cozy";
