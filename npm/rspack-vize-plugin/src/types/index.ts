/** Type definitions for @vizejs/rspack-plugin. */

// Native API Types

export interface SfcCompileOptionsNapi {
  filename?: string;
  sourceMap?: boolean;
  ssr?: boolean;
  /** Enable Vapor mode compilation */
  vapor?: boolean;
  /** Preserve TypeScript in output when true */
  isTs?: boolean;
  /** Scope ID for scoped CSS (e.g., "data-v-abc123") */
  scopeId?: string;
}

export interface SfcCompileResultNapi {
  code: string;
  css?: string;
  /** Source map JSON (when implemented) */
  map?: string;
  errors: string[];
  warnings: string[];
}

// CSS Compile API Types

export interface CssCompileTargets {
  chrome?: number;
  firefox?: number;
  safari?: number;
  edge?: number;
  ios?: number;
  android?: number;
}

export interface CssCompileOptions {
  /** Filename for error reporting */
  filename?: string;
  /** Whether to apply scoped CSS transformation */
  scoped?: boolean;
  /**
   * Scope ID for scoped CSS. Must be the full attribute (e.g., "data-v-abc123").
   */
  scopeId?: string;
  /** Whether to generate source maps */
  sourceMap?: boolean;
  /** Whether to minify the output */
  minify?: boolean;
  /** Whether to enable custom media query resolution */
  customMedia?: boolean;
  /** Browser targets for autoprefixing */
  targets?: CssCompileTargets;
}

export interface CssCompileResult {
  /** Compiled CSS code */
  code: string;
  /** Source map (null until implemented) */
  map?: string | null;
  /** CSS variables found (v-bind() expressions) */
  cssVars: string[];
  /** Errors during compilation */
  errors: string[];
  /** Warnings during compilation */
  warnings: string[];
}

// Style Block Types

export interface StyleBlockInfo {
  /** Raw style content */
  content: string;
  /** External source from `<style src="...">` */
  src?: string | null;
  /** Language of the style block (e.g., "css", "scss", "less", "sass", "stylus") */
  lang: string | null;
  /** Whether scoped */
  scoped: boolean;
  /** CSS Modules: true for unnamed, or binding name for named */
  module: boolean | string;
  /** Block index in the SFC */
  index: number;
}

// Custom Block Types

export interface CustomBlockInfo {
  /** Tag name (e.g., "i18n", "docs") */
  type: string;
  /** Raw content */
  content: string;
  /** External source from `<block src="...">` */
  src?: string | null;
  /** All attributes on the tag */
  attrs: Record<string, string | true>;
  /** Block index in the SFC */
  index: number;
}

// SFC Block Src Info

export interface SfcSrcInfo {
  /** <script src> path, or null */
  scriptSrc?: string | null;
  /** <template src> path, or null */
  templateSrc?: string | null;
}

// Template Asset URL Types

/** Static asset URL in template to be rewritten as an import binding. */
export interface TemplateAssetUrl {
  /** Raw URL as in template (e.g., "./logo.png") */
  url: string;
  /** JS identifier for the import (e.g., "_imports_0") */
  varName: string;
}

// Compiled Module Types

export interface CompiledModule {
  code: string;
  css?: string;
  errors: string[];
  warnings: string[];
  scopeId: string;
  hasScoped: boolean;
  /** Per-block style metadata */
  styles: StyleBlockInfo[];
  /** Custom blocks from the SFC */
  customBlocks: CustomBlockInfo[];
  /** Whether custom element (e.g., .ce.vue) */
  isCustomElement: boolean;
  /** Static asset URLs needing import rewrite. Empty when transformAssetUrls is false. */
  templateAssetUrls: TemplateAssetUrl[];
}

// Loader Options Types

export interface VizeLoaderOptions {
  /** Source maps @default true */
  sourceMap?: boolean;

  /** SSR mode @default false */
  ssr?: boolean;

  /** Project root */
  root?: string;

  /** Include filter */
  include?: string | RegExp | (string | RegExp)[];

  /** Exclude filter */
  exclude?: string | RegExp | (string | RegExp)[];

  /** Low-level compiler options for @vizejs/native compileSfc */
  compilerOptions?: SfcCompileOptionsNapi;

  /** Custom element mode. true=all, RegExp=matched. @default /\.ce\.vue$/ */
  customElement?: boolean | RegExp;

  /** Vapor mode @default false */
  vapor?: boolean;

  /** HMR. false to disable in dev. @default true (dev), false (prod/SSR) */
  hotReload?: boolean;

  /**
   * Transform static asset URLs in templates into import bindings.
   * true=built-in tags, false=disabled, object=custom map. @default true
   */
  transformAssetUrls?: boolean | Record<string, string[]>;
}

export interface VizeStyleLoaderOptions {
  /** Native CSS mode (experiments.css) @default false */
  native?: boolean;
}

// Plugin Options Types

export interface VizeRspackPluginOptions {
  /** Include filter @default /\.vue$/ */
  include?: string | RegExp | (string | RegExp)[];

  /** Exclude filter @default /node_modules/ */
  exclude?: string | RegExp | (string | RegExp)[];

  /** Force production mode @default auto-detected */
  isProduction?: boolean;

  /** SSR mode @default false */
  ssr?: boolean;

  /** Source maps @default true (dev), false (prod) */
  sourceMap?: boolean;

  /** Vapor mode @default false */
  vapor?: boolean;

  /** Root directory @default Rspack's root */
  root?: string;

  /** CSS config */
  css?: {
    /** Native CSS (experiments.css), uses LightningCSS @default false */
    native?: boolean;
  };

  /** Compiler options */
  compilerOptions?: SfcCompileOptionsNapi;

  /** Debug logging @default false */
  debug?: boolean;

  /** Auto-clone CSS rules for Vue style sub-requests (like VueLoaderPlugin). @default true */
  autoRules?: boolean;

  /**
   * Auto-inject a `builtin:swc-loader` post-processing rule to strip TypeScript
   * annotations from compiled `.vue` output. Safe for non-TS SFCs (SWC passes
   * plain JS through unchanged). Set to `false` to handle TS stripping yourself.
   * @default true
   */
  typescript?: boolean;
}

// Utility Types

/** Loader entry: either a string (loader name/path) or an object with loader + options */
export type LoaderEntry = string | { loader: string; options?: Record<string, unknown> };
