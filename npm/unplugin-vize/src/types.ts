export interface SfcCompileOptionsNapi {
  filename?: string;
  mode?: "module" | "function";
  sourceMap?: boolean;
  ssr?: boolean;
  vapor?: boolean;
  customRenderer?: boolean;
  templateSyntax?: VizeTemplateSyntax;
  runtimeModuleName?: string;
  runtimeGlobalName?: string;
  vueVersion?: VizeVueVersion;
  scopeId?: string;
}

export interface JsxCompileOptionsNapi {
  filename?: string;
  lang?: "jsx" | "tsx";
  /** Default JSX output mode; mirrors `compiler.jsxMode`, wins over `vapor`. */
  jsxMode?: "vdom" | "vapor";
  vapor?: boolean;
  /** Emit a v3 source map for the generated render code (#1533). */
  sourceMap?: boolean;
}

/** A JSX component's extracted `<style scoped>` block (#1495, #1533). */
export interface JsxScopedStyleNapi {
  /** Generated scope id, e.g. `data-v-1a2b3c4d`, already applied to the CSS. */
  scopeId: string;
  /** Scope-rewritten CSS, with the `data-v-<hash>` attribute applied. */
  css: string;
}

export interface JsxCompileResultNapi {
  /**
   * Self-contained module: the deduplicated runtime-helper preamble followed by
   * every component's render code (the helper imports are no longer dropped,
   * #1533).
   */
  code: string;
  /**
   * v3 source map (JSON) for `code`, present only when `sourceMap` was requested
   * and the module is a single component. `null` otherwise (#1533).
   */
  map?: string;
  errors: string[];
  warnings: string[];
  /**
   * Extracted `<style scoped>` blocks across the module's components, in source
   * order (#1495). Empty when no component had a `<style scoped>`. Each entry's
   * CSS is already scope-rewritten; the plugin emits it through the same path
   * SFC `<style>` blocks use (#1533).
   */
  scopedStyles: JsxScopedStyleNapi[];
}

export interface MacroArtifact {
  kind: string;
  name: string;
  source: string;
  content: string;
  moduleCode?: string;
  start: number;
  end: number;
}

export interface SfcCompileResultNapi {
  code: string;
  css?: string;
  errors: string[];
  warnings: string[];
  templateHash?: string;
  styleHash?: string;
  scriptHash?: string;
  hasScoped: boolean;
  styles: StyleBlockNapi[];
  macroArtifacts?: MacroArtifact[];
}

export interface StyleBlockNapi {
  content: string;
  src?: string;
  lang?: string;
  scoped: boolean;
  module: boolean;
  moduleName?: string;
  index: number;
}

export interface VizeUnpluginOptions {
  include?: string | RegExp | Array<string | RegExp>;
  exclude?: string | RegExp | Array<string | RegExp>;
  compatibility?: VizeCompatibilityOptions;
  isProduction?: boolean;
  ssr?: boolean;
  sourceMap?: boolean;
  mode?: "module" | "function";
  vapor?: boolean;
  /**
   * Default output mode for `.jsx`/`.tsx` components without a `"use vue:*"`
   * directive. Distinct from `vapor` (which targets `.vue` SFCs). A
   * per-component directive overrides it.
   * @default "vdom"
   */
  jsxMode?: "vdom" | "vapor";
  customRenderer?: boolean;
  templateSyntax?: VizeTemplateSyntax;
  runtimeModuleName?: string;
  runtimeGlobalName?: string;
  vueVersion?: VizeVueVersion;
  root?: string;
  debug?: boolean;
}

export type VizeVueVersion = 0.11 | 1 | 2 | 3 | "legacy";
export type VizeTemplateSyntax = "standard" | "strict" | "quirks";

export interface VizeCompatibilityOptions {
  /**
   * Host Vue version. Vue 0.11/1/2 opt into host-compiler compatibility.
   */
  vueVersion?: VizeVueVersion;
  /**
   * Keep .vue files on the existing Vue compiler for legacy Vue runtimes.
   * @default true when vueVersion is 0.11, 1, 2, or "legacy"
   */
  hostCompiler?: boolean;
  /**
   * Enable function-body output for CDN/global Vue evaluation.
   */
  scriptSetupInStandalone?: boolean;
  /**
   * Allow Vapor output for Options API SFCs when vapor is enabled.
   */
  optionsApiVapor?: boolean;
  /**
   * Override the host Nuxt major when this option object is shared with Nuxt.
   */
  nuxtVersion?: 2 | 3 | 4;
  /**
   * Force Webpack compatibility behavior.
   *
   * Webpack 4 does not expose `compiler.webpack`, so the plugin resolves
   * `DefinePlugin` from the host `webpack` package when this is `4` or when
   * auto-detection sees a Webpack 4 compiler shape.
   */
  webpackVersion?: 4 | 5;
}

export interface StyleBlockInfo {
  content: string;
  src?: string | null;
  lang: string | null;
  scoped: boolean;
  module: boolean | string;
  index: number;
}

export interface CompiledModule {
  code: string;
  css?: string;
  scopeId: string;
  hasScoped: boolean;
  templateHash?: string;
  styleHash?: string;
  scriptHash?: string;
  macroArtifacts?: MacroArtifact[];
  styles: StyleBlockInfo[];
}

export interface CachedCompiledModule {
  compiled: CompiledModule;
  sourceHash: string;
  signature: string;
}

export interface NormalizedVizeUnpluginOptions {
  include?: string | RegExp | Array<string | RegExp>;
  exclude?: string | RegExp | Array<string | RegExp>;
  compatibility: VizeCompatibilityOptions;
  isProduction: boolean;
  ssr: boolean;
  sourceMap: boolean;
  mode: "module" | "function";
  vapor: boolean;
  /** Default JSX output mode; `undefined` when unset (treated as VDOM). */
  jsxMode?: "vdom" | "vapor";
  customRenderer: boolean;
  templateSyntax: VizeTemplateSyntax;
  runtimeModuleName: string;
  runtimeGlobalName: string;
  vueVersion: VizeVueVersion;
  hostCompiler: boolean;
  root: string;
  debug: boolean;
}

export interface ParsedVueRequestQuery {
  vue: boolean;
  type: string | null;
  index: number | null;
  lang: string | null;
  module: boolean | string;
  scoped: string | null;
  vizeFile: string | null;
}

export interface ParsedVueRequest {
  filename: string;
  path: string;
  query: ParsedVueRequestQuery;
}
