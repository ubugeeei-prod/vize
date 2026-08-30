import "./theme.css";
import "./theme-preset-headless.css";
import "./theme-preset-atelier.css";
import "./theme-preset-midnight.css";
import "./theme-preset-paper.css";
import "./theme-preset-play.css";
import "./theme-preset-signal.css";
import "./theme-preset-high-contrast.css";

import { setThemeTokens as setThemeTokensBase } from "./theme-tokens.ts";
import type { ThemeTokenOverrides } from "./theme-types.ts";

/** Override theme tokens on one element's subtree. */
export function setThemeTokens(
  element: ElementCSSInlineStyle,
  overrides: ThemeTokenOverrides,
): () => void {
  return setThemeTokensBase(element, overrides);
}

export {
  themeCascadeLayerOrder,
  themeDensityAttribute,
  themeDensityScales,
  themePresetAttribute,
  themePresets,
  themeTokenPackNames,
  themeTokenProperty,
  themeTokens,
  themeTokensForPack,
  themeTokenVar,
} from "./theme-tokens.ts";

export type {
  ThemeBorderToken,
  ThemeColorToken,
  ThemeDensityScale,
  ThemeElevationToken,
  ThemeFeedbackToneToken,
  ThemeFocusToken,
  ThemeOpacityToken,
  ThemePresetName,
  ThemeRadiusToken,
  ThemeSizeToken,
  ThemeSpaceToken,
  ThemeTokenPackName,
  ThemeTokenName,
  ThemeTokenOverrides,
  ThemeTypeToken,
  ThemeZIndexToken,
} from "./theme-types.ts";
