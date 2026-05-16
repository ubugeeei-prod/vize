import type { VrtOptions } from "./vrt.js";

/**
 * Theme color definitions for Musea gallery UI.
 * All properties are optional — unspecified colors inherit from the `base` built-in theme.
 */
export interface MuseaThemeColors {
  bgPrimary?: string;
  bgSecondary?: string;
  bgTertiary?: string;
  bgElevated?: string;
  accent?: string;
  accentHover?: string;
  accentContrast?: string;
  accentSubtle?: string;
  text?: string;
  textSecondary?: string;
  textMuted?: string;
  border?: string;
  borderSubtle?: string;
  success?: string;
  error?: string;
  info?: string;
  warning?: string;
  shadow?: string;
}

/**
 * Custom theme definition.
 */
export interface MuseaTheme {
  /** Unique name for this theme. */
  name: string;
  /** Built-in theme to inherit unspecified colors from. @default 'dark' */
  base?: "dark" | "light";
  /** Color overrides. */
  colors: MuseaThemeColors;
}

/**
 * Musea plugin options.
 */
export interface MuseaOptions {
  /**
   * Glob patterns to include Art files.
   * @default ['**\/*.art.vue']
   */
  include?: string[];

  /**
   * Glob patterns to exclude.
   * @default ['node_modules/**', 'dist/**']
   */
  exclude?: string[];

  /**
   * Base path for Musea gallery UI.
   * @default '/__musea__'
   */
  basePath?: string;

  /**
   * Enable Storybook CSF output.
   * @default false
   */
  storybookCompat?: boolean;

  /**
   * Storybook output directory (when storybookCompat is true).
   * @default '.storybook/stories'
   */
  storybookOutDir?: string;

  /**
   * Enable inline <art> blocks in regular .vue SFC files.
   * When enabled, regular .vue files containing <art> blocks will be
   * included in the gallery. Use <Self> to reference the host component.
   * @default false
   */
  inlineArt?: boolean;

  /**
   * VRT (Visual Regression Testing) configuration.
   */
  vrt?: VrtOptions;

  /**
   * Path to Style Dictionary tokens JSON file or directory.
   * Supports standard Style Dictionary format.
   * @example 'src/tokens.json' or 'src/tokens/'
   */
  tokensPath?: string;

  /**
   * Gallery theme configuration.
   *
   * - `'dark'` / `'light'` — use a built-in theme (default: `'dark'`)
   * - `'system'` — follow the OS color-scheme preference
   * - `MuseaTheme` — single custom theme (replaces defaults)
   * - `MuseaTheme[]` — multiple custom themes (first is default, user can switch)
   */
  theme?: "dark" | "light" | "system" | MuseaTheme | MuseaTheme[];

  /**
   * CSS files to inject into component preview iframes.
   * Useful for loading global styles (custom properties, resets, fonts, etc.)
   * that components depend on.
   *
   * Paths are resolved relative to the project root.
   * @example ['app/assets/styles/main.css']
   */
  previewCss?: string[];

  /**
   * Path to a module that exports a default setup function for preview iframes.
   * The function receives the Vue `App` instance and can install plugins
   * (e.g. vue-i18n, vue-router) before the component is mounted.
   *
   * Signature: `(app: App) => void | Promise<void>`
   *
   * Path is resolved relative to the project root.
   * @example 'musea.preview.ts'
   */
  previewSetup?: string;
}
