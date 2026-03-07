/**
 * Type definitions for @vizejs/rspack-plugin
 * Copied and adapted from vite-plugin-vize
 */

// ============================================================================
// Native API Types
// ============================================================================

export interface SfcCompileOptionsNapi {
  filename?: string;
  sourceMap?: boolean;
  ssr?: boolean;
  /** Preserve TypeScript in output when true */
  isTs?: boolean;
  /** Scope ID for scoped CSS (e.g., "data-v-abc123") */
  scopeId?: string;
}

export interface SfcCompileResultNapi {
  code: string;
  css?: string;
  /** Source map JSON string (when implemented in @vizejs/native) */
  map?: string;
  errors: string[];
  warnings: string[];
}

// ============================================================================
// Style Block Types
// ============================================================================

export interface StyleBlockInfo {
  /** Raw style content (uncompiled for preprocessor langs) */
  content: string;
  /** External style source path from `<style src="...">` */
  src?: string | null;
  /** Language of the style block (e.g., "css", "scss", "less", "sass", "stylus") */
  lang: string | null;
  /** Whether the style block has the scoped attribute */
  scoped: boolean;
  /** CSS Modules: true for unnamed `module`, or the binding name for `module="name"` */
  module: boolean | string;
  /** Index of this style block in the SFC */
  index: number;
}

// ============================================================================
// Custom Block Types
// ============================================================================

export interface CustomBlockInfo {
  /** Tag name of the custom block (e.g., "i18n", "docs") */
  type: string;
  /** Raw content of the custom block */
  content: string;
  /** External source path from `<block src="...">` */
  src?: string | null;
  /** All attributes on the custom block tag */
  attrs: Record<string, string | true>;
  /** Index of this custom block in the SFC */
  index: number;
}

// ============================================================================
// SFC Block Src Info
// ============================================================================

export interface SfcSrcInfo {
  /** Whether <script> has a src attribute */
  scriptSrc?: string | null;
  /** Whether <template> has a src attribute */
  templateSrc?: string | null;
}

// ============================================================================
// Compiled Module Types
// ============================================================================

export interface CompiledModule {
  code: string;
  css?: string;
  errors: string[];
  warnings: string[];
  scopeId: string;
  hasScoped: boolean;
  /** Per-block style metadata extracted from the source SFC */
  styles: StyleBlockInfo[];
  /** Custom blocks extracted from the source SFC */
  customBlocks: CustomBlockInfo[];
  /** Whether this is a custom element SFC (e.g., .ce.vue) */
  isCustomElement: boolean;
}

// ============================================================================
// Loader Options Types
// ============================================================================

export interface VizeLoaderOptions {
  /**
   * Enable source map generation
   * @default true
   */
  sourceMap?: boolean;

  /**
   * Enable SSR mode
   * @default false
   */
  ssr?: boolean;

  /**
   * Project root directory
   */
  root?: string;

  /**
   * Files to include in compilation (safe filter)
   */
  include?: string | RegExp | (string | RegExp)[];

  /**
   * Files to exclude from compilation (safe filter)
   */
  exclude?: string | RegExp | (string | RegExp)[];

  /**
   * Additional low-level compiler options passed to @vizejs/native compileSfc
   */
  compilerOptions?: SfcCompileOptionsNapi;

  /**
   * Transform Vue SFCs into custom elements.
   * - `true`: all `*.vue` imports are converted into custom elements
   * - `RegExp`: matched files are converted into custom elements
   *
   * @default /\.ce\.vue$/
   */
  customElement?: boolean | RegExp;

  /**
   * Enable HMR (Hot Module Replacement) for Vue SFCs.
   * Set to `false` to explicitly disable HMR even in development mode.
   *
   * @default true (enabled in development, disabled in production/SSR)
   */
  hotReload?: boolean;
}

export interface VizeStyleLoaderOptions {
  /**
   * Whether to use native CSS mode (experiments.css)
   * In both modes, the loader outputs pure CSS
   * @default false
   */
  native?: boolean;
}

// ============================================================================
// Preset API Types
// ============================================================================

export type VizeStyleLanguage = "css" | "scss" | "sass" | "less" | "styl" | "stylus";

export interface CreateVizeVueRulesOptions {
  /**
   * Production mode flag
   * @default false
   */
  isProduction?: boolean;

  /**
   * Use Rspack native CSS pipeline (`experiments.css`)
   * @default true
   */
  nativeCss?: boolean;

  /**
   * Extra languages to handle automatically in addition to `css`
   * @default ["scss", "sass", "less", "stylus", "styl"]
   */
  styleLanguages?: VizeStyleLanguage[];

  /**
   * Loader entry for the main `.vue` compiler loader
   * @default "@vizejs/rspack-plugin/loader"
   */
  vizeLoader?: string;

  /**
   * Loader entry for the `.vue` style extractor loader
   * @default "@vizejs/rspack-plugin/style-loader"
   */
  vizeStyleLoader?: string;

  /**
   * Dev style injector / prod extractor loader entry used in non-native CSS mode
   * @default "style-loader"
   */
  styleInjectLoader?: LoaderEntry;

  /**
   * Production CSS extractor loader entry used in non-native CSS mode
   * (e.g. `rspack.CssExtractRspackPlugin.loader`)
   */
  styleExtractLoader?: LoaderEntry;

  /**
   * css-loader entry used in non-native CSS mode
   * @default "css-loader"
   */
  cssLoader?: LoaderEntry;

  /**
   * Main vize loader options
   */
  loaderOptions?: VizeLoaderOptions;

  /**
   * Vize style loader options
   */
  styleLoaderOptions?: VizeStyleLoaderOptions;

  /**
   * Add a post-processing rule to strip TypeScript type annotations from
   * the compiled `.vue` output.
   *
   * This is needed because `@vizejs/native compileSfc` preserves TypeScript
   * syntax in its output (same as `@vue/compiler-sfc`), and a downstream
   * transpiler must strip the types before the browser/runtime can execute it.
   *
   * - `true`:  use Rspack built-in SWC loader (`builtin:swc-loader`)
   * - `LoaderEntry`: use a custom loader (e.g. `esbuild-loader`)
   * - `false` / omitted: no post-processing (user handles it separately)
   *
   * @default false
   */
  typescript?: boolean | LoaderEntry;
}

// ============================================================================
// Plugin Options Types
// ============================================================================

export interface VizeRspackPluginOptions {
  /**
   * Files to include in compilation
   * @default /\.vue$/
   */
  include?: string | RegExp | (string | RegExp)[];

  /**
   * Files to exclude from compilation
   * @default /node_modules/
   */
  exclude?: string | RegExp | (string | RegExp)[];

  /**
   * Force production mode
   * @default auto-detected from Rspack config
   */
  isProduction?: boolean;

  /**
   * Enable SSR mode
   * @default false
   */
  ssr?: boolean;

  /**
   * Enable source map generation
   * @default true in development, false in production
   */
  sourceMap?: boolean;

  /**
   * Enable Vapor mode compilation
   * @default false
   */
  vapor?: boolean;

  /**
   * Root directory to scan for .vue files
   * @default Rspack's root
   */
  root?: string;

  /**
   * CSS configuration
   */
  css?: {
    /**
     * Use Rspack native CSS processing (experiments.css)
     * When enabled, no need for css-loader/style-loader/CssExtractRspackPlugin
     * Handled by Rust-side LightningCSS, better performance
     * @default false
     */
    native?: boolean;
  };

  /**
   * Custom compiler options
   */
  compilerOptions?: SfcCompileOptionsNapi;

  /**
   * Enable debug logging
   * @default false
   */
  debug?: boolean;
}

// ============================================================================
// Utility Types
// ============================================================================

/** Loader entry: either a string (loader name/path) or an object with loader + options */
export type LoaderEntry = string | { loader: string; options?: Record<string, unknown> };
