/** Public adapter types; native compiler types remain re-exported here. */

import type { MacroArtifact, SfcCompileOptionsNapi } from "./native.ts";
export type * from "./native.ts";

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
  /** Per-source metadata for browser-local HMR comparisons. */
  hmr?: {
    source: string;
    script?: string;
    template?: string;
    options: string;
    canRerender: boolean;
  };
  /** Source map v3 JSON for the native module, before output assembly. */
  map?: string;
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
  /** Compile-time macro artifacts extracted from the source SFC. */
  macroArtifacts?: MacroArtifact[];
}

// Loader Options Types

/** Native options with loader-level aliases retained for existing configurations. */
export type VizeSfcCompilerOptions = Omit<SfcCompileOptionsNapi, "ssr" | "vapor" | "sourceMap"> & {
  /** @deprecated Use the SFC loader's top-level ssr option. */
  ssr?: boolean;
  /** @deprecated Use the SFC loader's top-level vapor option. */
  vapor?: boolean;
  /** @deprecated Use the SFC loader's top-level sourceMap option. */
  sourceMap?: boolean;
};

/** Compatibility type; prefer VizeSfcLoaderOptions or VizeJsxLoaderOptions. */
export interface VizeLoaderOptions {
  /** Override production output; otherwise follows loader mode / NODE_ENV. */
  isProduction?: boolean;
  /** Source maps; falls back to compilerOptions, then Rspack's loader context. */
  sourceMap?: boolean;

  /** SSR mode @default false */
  ssr?: boolean;

  /** Project root */
  root?: string;

  /** @deprecated Use module.rules[].include. Retained as a loader passthrough filter. */
  include?: string | RegExp | (string | RegExp)[];

  /** @deprecated Use module.rules[].exclude. Retained as a loader passthrough filter. */
  exclude?: string | RegExp | (string | RegExp)[];

  /** Low-level compiler options for @vizejs/native compileSfc */
  compilerOptions?: VizeSfcCompilerOptions;

  /** Custom element mode. true=all, RegExp=matched. @default /\.ce\.vue$/ */
  customElement?: boolean | RegExp;

  /** Vapor mode @default false */
  vapor?: boolean;

  /** Default JSX output mode for `.jsx`/`.tsx` without a `"use vue:*"` directive. @default "vdom" */
  jsxMode?: "vdom" | "vapor";

  /** JSX semantics; `"babel"` opts into @vue/babel-plugin-jsx compatibility. */
  jsxCompat?: "native" | "babel";

  /** HMR. false to disable in dev. @default true (dev), false (prod/SSR) */
  hotReload?: boolean;

  /** CSS handling config */
  css?: {
    /** With autoRules: false, override the plugin default for this SFC chain.
     * With automatic rules, must match the plugin's resolved CSS mode. */
    native?: boolean;
  };

  /**
   * Transform static asset URLs in templates into import bindings.
   * true=built-in tags, false=disabled, object=custom map. @default true
   */
  transformAssetUrls?: boolean | Record<string, string[]>;
}

/** Options accepted by the SFC loader. JSX compilation uses a separate loader. */
export type VizeSfcLoaderOptions = Omit<VizeLoaderOptions, "jsxMode" | "jsxCompat">;

/** Options accepted by the JSX/TSX loader. */
export type VizeJsxLoaderOptions = Pick<
  VizeLoaderOptions,
  "sourceMap" | "vapor" | "jsxMode" | "jsxCompat" | "include" | "exclude"
>;

export interface VizeStyleLoaderOptions {
  /** Rspack native CSS mode @default false */
  native?: boolean;
}

// Plugin Options Types

export interface VizeRspackPluginOptions {
  /** @deprecated Use rule.include. This legacy field only filters plugin watch logs. */
  include?: string | RegExp | (string | RegExp)[];

  /** @deprecated Use rule.exclude. This legacy field only filters plugin watch logs. */
  exclude?: string | RegExp | (string | RegExp)[];

  /** @deprecated Use Rspack mode. Retained for plugin flags/logging only. */
  isProduction?: boolean;

  /** @deprecated Use SFC loader ssr. This plugin field is not forwarded. */
  ssr?: boolean;

  /** @deprecated Use loader sourceMap. This plugin field is not forwarded. */
  sourceMap?: boolean;

  /** @deprecated Use loader vapor. This plugin field only affects the legacy debug message. */
  vapor?: boolean;

  /** @deprecated Use JSX loader jsxMode. This plugin field is not forwarded. */
  jsxMode?: "vdom" | "vapor";

  /** @deprecated Use JSX loader jsxCompat. This plugin field is not forwarded. */
  jsxCompat?: "native" | "babel";

  /** @deprecated Use loader root or Rspack context. This plugin field is not forwarded. */
  root?: string;

  /** CSS config */
  css?: {
    /** Automatic CSS mode, or default for manual SFC rules. Auto-detected from
     * the Rspack version and experiments.css when omitted. */
    native?: boolean;
  };

  /** @deprecated Use SFC loader compilerOptions. This plugin field is not forwarded. */
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
