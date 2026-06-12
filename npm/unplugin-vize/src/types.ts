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
  vapor?: boolean;
}

export interface JsxCompileResultNapi {
  code: string;
  errors: string[];
  warnings: string[];
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
