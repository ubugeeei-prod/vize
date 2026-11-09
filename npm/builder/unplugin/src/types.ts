/** Native SFC compiler options forwarded by the bundler adapters. */
export interface SfcCompileOptionsNapi {
  filename?: string;
  mode?: "module" | "function";
  sourceMap?: boolean;
  ssr?: boolean;
  vapor?: boolean;
  customRenderer?: boolean;
  customElements?: string[];
  templateSyntax?: VizeTemplateSyntax;
  experimentalInTagComments?: boolean;
  experimentalPatternedTemplate?: boolean;
  experimentalSelfComponent?: boolean;
  experimentalStrictSlotChildren?: boolean;
  experimentalServerScript?: boolean;
  runtimeModuleName?: string;
  runtimeGlobalName?: string;
  vueVersion?: VizeVueVersion;
  scopeId?: string;
}

/** Native JSX compiler options forwarded by the bundler adapters. */
export interface JsxCompileOptionsNapi {
  filename?: string;
  lang?: "jsx" | "tsx";
  /** Default JSX output mode; mirrors `compiler.jsxMode`, wins over `vapor`. */
  jsxMode?: "vdom" | "vapor";
  /** JSX semantics; `"babel"` opts into @vue/babel-plugin-jsx compatibility. */
  jsxCompat?: "native" | "babel";
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

/** Native JSX compiler output plus diagnostics and extracted scoped styles. */
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

/** Generated compiler-macro artifact associated with one authored source range. */
export interface MacroArtifact {
  kind: string;
  name: string;
  source: string;
  content: string;
  moduleCode?: string;
  start: number;
  end: number;
}

/** Native SFC compiler output consumed by every host adapter. */
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

/** One compiled SFC style block returned by the native compiler. */
export interface StyleBlockNapi {
  content: string;
  src?: string;
  lang?: string;
  scoped: boolean;
  module: boolean;
  moduleName?: string;
  index: number;
}

/** Public options shared by every `@vizejs/unplugin` host adapter. */
export interface VizeUnpluginOptions {
  /** Source patterns eligible for compilation. @default [/\.vue$/, /\.[jt]sx$/] */
  include?: string | RegExp | Array<string | RegExp>;
  /** Source patterns excluded from compilation. @default [/node_modules/] */
  exclude?: string | RegExp | Array<string | RegExp>;
  /** Host-runtime compatibility overrides. @default {} */
  compatibility?: VizeCompatibilityOptions;
  /** Emit production-oriented runtime code. @default process.env.NODE_ENV === "production" */
  isProduction?: boolean;
  /** Compile for server rendering. @default false */
  ssr?: boolean;
  /** Emit source maps. @default !isProduction */
  sourceMap?: boolean;
  /** Compiler output form. @default "module" */
  mode?: "module" | "function";
  /** Compile Vue SFCs for the Vapor runtime. @default false */
  vapor?: boolean;
  /** Enable experimental in-tag compiler comments. @default false */
  experimentalInTagComments?: boolean;
  /** Enable experimental patterned-template compilation. @default false */
  experimentalPatternedTemplate?: boolean;
  /** Enable experimental self-component resolution. @default false */
  experimentalSelfComponent?: boolean;
  /** Enable experimental strict slot children. @default false */
  experimentalStrictSlotChildren?: boolean;
  /** Enable experimental server-script blocks. @default false */
  experimentalServerScript?: boolean;
  /**
   * Default output mode for `.jsx`/`.tsx` components without a `"use vue:*"`
   * directive. Distinct from `vapor` (which targets `.vue` SFCs). A
   * per-component directive overrides it.
   * @default "vdom"
   */
  jsxMode?: "vdom" | "vapor";
  /** JSX semantics; `"babel"` opts into @vue/babel-plugin-jsx compatibility. @default "native" */
  jsxCompat?: "native" | "babel";
  /** Compile against the custom-renderer runtime contract. @default false */
  customRenderer?: boolean;
  /** Tag names compiled as custom elements. @default [] */
  customElements?: string[];
  /** Template parsing compatibility mode. @default "standard" */
  templateSyntax?: VizeTemplateSyntax;
  /** Module specifier used for Vue runtime helpers. @default "vue" */
  runtimeModuleName?: string;
  /** Global identifier used by function-mode output. @default "Vue" */
  runtimeGlobalName?: string;
  /** Host Vue compatibility target. @default 3 */
  vueVersion?: VizeVueVersion;
  /** Project root used to resolve style and source dependencies. @default process.cwd() */
  root?: string;
  /** Emit adapter debug details. @default false */
  debug?: boolean;
}

/** Vue runtime versions supported by the bundler compatibility layer. */
export type VizeVueVersion = 0.11 | 1 | 2 | "2.7" | 3 | "legacy";
/** Template parsing modes supported by the native compiler. */
export type VizeTemplateSyntax = "standard" | "strict" | "quirks";

/** Host compiler and runtime compatibility overrides. */
export interface VizeCompatibilityOptions {
  /**
   * Host Vue version. Vue 0.11/1/2/2.7 opt into host-compiler compatibility.
   * @default 3
   */
  vueVersion?: VizeVueVersion;
  /**
   * Keep .vue files on the existing Vue compiler for legacy Vue runtimes.
   * @default true when vueVersion is 0.11, 1, 2, "2.7", or "legacy"
   */
  hostCompiler?: boolean;
  /**
   * Enable function-body output for CDN/global Vue evaluation.
   * @default false
   */
  scriptSetupInStandalone?: boolean;
  /**
   * Allow Vapor output for Options API SFCs when vapor is enabled.
   * @default false
   */
  optionsApiVapor?: boolean;
  /**
   * Override the host Nuxt major when this option object is shared with Nuxt.
   * @default undefined
   */
  nuxtVersion?: 2 | 3 | 4 | 5;
  /**
   * Force Webpack compatibility behavior.
   *
   * Webpack 4 does not expose `compiler.webpack`, so the plugin resolves
   * `DefinePlugin` from the host `webpack` package when this is `4` or when
   * auto-detection sees a Webpack 4 compiler shape.
   *
   * @default undefined
   */
  webpackVersion?: 4 | 5;
}

/** Normalized metadata for one authored or external SFC style block. */
export interface StyleBlockInfo {
  content: string;
  src?: string | null;
  /** The file an inlined `<style src>` block was read from (#11522). */
  srcFile?: string | null;
  lang: string | null;
  scoped: boolean;
  module: boolean | string;
  index: number;
}

/** Host-independent compiled module retained by the adapter cache. */
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

/** Cached compiled module bound to its source hash and option signature. */
export interface CachedCompiledModule {
  compiled: CompiledModule;
  sourceHash: string;
  signature: string;
}

/** Fully resolved adapter options used after public option normalization. */
export interface NormalizedVizeUnpluginOptions {
  /** Resolved include filter; omission retains the built-in Vue/JSX patterns. @default undefined */
  include?: string | RegExp | Array<string | RegExp>;
  /** Resolved exclude filter; omission retains the node_modules exclusion. @default undefined */
  exclude?: string | RegExp | Array<string | RegExp>;
  compatibility: VizeCompatibilityOptions;
  isProduction: boolean;
  ssr: boolean;
  sourceMap: boolean;
  mode: "module" | "function";
  vapor: boolean;
  experimentalInTagComments: boolean;
  experimentalPatternedTemplate: boolean;
  experimentalSelfComponent: boolean;
  experimentalStrictSlotChildren: boolean;
  experimentalServerScript: boolean;
  /** Default JSX output mode; `undefined` when unset (treated as VDOM). @default undefined */
  jsxMode?: "vdom" | "vapor";
  /** JSX semantics; `undefined` when unset (treated as native). @default undefined */
  jsxCompat?: "native" | "babel";
  customRenderer: boolean;
  /** Custom element tag names, or `undefined` when none were configured. @default undefined */
  customElements?: string[];
  templateSyntax: VizeTemplateSyntax;
  runtimeModuleName: string;
  runtimeGlobalName: string;
  vueVersion: VizeVueVersion;
  hostCompiler: boolean;
  root: string;
  debug: boolean;
}

/** What the host bundler does with a stylesheet a module imports. */
export type HostCss =
  /** Bundles none (Babel, esbuild without `bundle`): plain CSS is injected. */
  | "none"
  /** Bundles CSS but compiles no preprocessor language (esbuild). */
  | "css"
  /** Its plugins also compile Sass, Less and Stylus (Rollup, Rolldown, webpack). */
  | "preprocessors";

/** Parsed query fields for one Vue virtual-module request. */
export interface ParsedVueRequestQuery {
  vue: boolean;
  type: string | null;
  index: number | null;
  /** The language the style module holds, from its `lang.<ext>` key. */
  lang: string | null;
  module: boolean | string;
  scoped: string | null;
}

/** Physical filename and normalized query for one Vue module request. */
export interface ParsedVueRequest {
  filename: string;
  query: ParsedVueRequestQuery;
}
