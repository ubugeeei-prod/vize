import type { VrtOptions } from "./vrt.js";
import type { MuseaTokenPreviewConfig } from "../tokens/preview.js";

export type MuseaVueVersion = 0.11 | 1 | 2 | "2.7" | 3 | "legacy";

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
   * Path to a Style Dictionary tokens JSON file/directory or Tailwind CSS theme file.
   * Supports standard Style Dictionary format and Tailwind v4 `@theme` CSS variables.
   * @example 'src/tokens.json', 'src/tokens/', or 'src/styles/main.css'
   */
  tokensPath?: string;

  /**
   * Design token preview rules for the gallery.
   * Custom rules run before Musea's built-in previews.
   */
  tokenPreviews?: MuseaTokenPreviewConfig;

  /**
   * Project root that should be treated as the outer boundary for path resolution
   * (notably `tokensPath`). When set, sibling directories of the Vite root that
   * live under this project root are allowed.
   *
   * Frameworks like Nuxt set this to their project root so a Vite app rooted at
   * `app/` can still load tokens from `<project>/design/tokens`.
   *
   * Defaults to the Vite root when unset.
   */
  projectRoot?: string;

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
   * Project-relative and `./` paths resolve against the project root.
   * Bare specifiers (`normalize.css`, `@fontsource/inter/index.css`) are
   * left for Vite to resolve.
   * @example ['app/assets/styles/main.css', 'normalize.css']
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

  /**
   * Host Vue version for preview runtime compatibility checks.
   * @default 3
   */
  vueVersion?: MuseaVueVersion;

  /**
   * Vue runtime compiler entry used by the static preview alias for bare `vue`.
   * Defaults to resolving `vue/dist/vue.esm-bundler.js` from the Vite project root.
   *
   * Use this when a workspace needs a custom Vue build or a non-standard package
   * manager layout.
   */
  vueRuntimeCompiler?: string;
}
