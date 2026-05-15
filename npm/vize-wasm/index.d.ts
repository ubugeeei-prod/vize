/**
 * Vize - WASM bindings
 *
 * This package provides WebAssembly bindings for the Vue compiler implemented in Rust.
 */

/** Compiler options for template compilation */
export interface CompilerOptions {
  /** Output mode: "module" or "function" */
  mode?: "module" | "function";
  /** Whether to prefix identifiers */
  prefixIdentifiers?: boolean;
  /** Whether to hoist static nodes */
  hoistStatic?: boolean;
  /** Whether to cache event handlers */
  cacheHandlers?: boolean;
  /** Scope ID for scoped CSS */
  scopeId?: string;
  /** Whether in SSR mode */
  ssr?: boolean;
  /** Whether to generate source map */
  sourceMap?: boolean;
  /** Filename for source map */
  filename?: string;
  /** Output mode: "vdom" or "vapor" */
  outputMode?: "vdom" | "vapor";
  /** Whether the template contains TypeScript */
  isTs?: boolean;
}

/** Result of template compilation */
export interface CompileResult {
  /** Generated code */
  code: string;
  /** Preamble code (imports) */
  preamble: string;
  /** AST (serialized) */
  ast: object;
  /** Source map */
  map?: object | null;
  /** Used helpers */
  helpers: string[];
  /** Template strings for Vapor mode static parts */
  templates?: string[];
}

/** SFC block (template, script, style) */
export interface SfcBlock {
  /** Block content */
  content: string;
  /** Source location */
  loc: { start: number; end: number };
  /** Language (e.g., "ts", "scss") */
  lang?: string;
  /** External source URL */
  src?: string;
  /** Block attributes */
  attrs: Record<string, string | true>;
}

/** SFC script block */
export interface SfcScriptBlock extends SfcBlock {
  /** Whether this is a setup script */
  setup: boolean;
}

/** SFC style block */
export interface SfcStyleBlock extends SfcBlock {
  /** Whether the style is scoped */
  scoped: boolean;
  /** CSS module name */
  module?: string | true;
}

/** Compile-time macro artifact extracted from an SFC */
export interface MacroArtifact {
  /** Stable artifact kind */
  kind: string;
  /** Macro call name */
  name: string;
  /** Full macro call source */
  source: string;
  /** Extracted macro payload source */
  content: string;
  /** Ready-to-load virtual module code */
  moduleCode?: string;
  /** Absolute start offset in the SFC source */
  start: number;
  /** Absolute end offset in the SFC source */
  end: number;
}

/** SFC descriptor (parsed .vue file) */
export interface SfcDescriptor {
  /** Filename */
  filename: string;
  /** Original source */
  source: string;
  /** Template block */
  template?: SfcBlock;
  /** Script block */
  script?: SfcScriptBlock;
  /** Script setup block */
  scriptSetup?: SfcScriptBlock;
  /** Style blocks */
  styles: SfcStyleBlock[];
  /** Custom blocks */
  customBlocks: Array<{
    type: string;
    content: string;
    attrs: Record<string, string | true>;
  }>;
}

/** SFC parse options */
export interface SfcParseOptions {
  /** Filename for the SFC */
  filename?: string;
}

/** SFC compile options */
export interface SfcCompileOptions extends CompilerOptions {
  /** Filename for the SFC */
  filename?: string;
}

/** Result of SFC compilation */
export interface SfcCompileResult {
  /** Parsed SFC descriptor */
  descriptor: SfcDescriptor;
  /** Compiled template result */
  template?: CompileResult;
  /** Compiled script result */
  script: {
    /** Generated JavaScript code */
    code: string;
    /** Binding metadata */
    bindings?: object;
  };
  /** Generated CSS */
  css?: string;
  /** Compilation errors */
  errors: string[];
  /** Compilation warnings */
  warnings: string[];
  /** Compile-time macro artifacts */
  macroArtifacts?: MacroArtifact[];
}

/** CSS compile options */
export interface CssCompileOptions {
  /** Scope ID for scoped CSS */
  scopeId?: string;
  /** Whether to scope the CSS */
  scoped?: boolean;
  /** Whether to minify */
  minify?: boolean;
  /** Whether to generate source map */
  sourceMap?: boolean;
  /** Filename for source map */
  filename?: string;
  /** Browser targets for CSS transformations */
  targets?: {
    chrome?: number;
    firefox?: number;
    safari?: number;
    edge?: number;
    ios?: number;
    android?: number;
  };
}

/** Result of CSS compilation */
export interface CssCompileResult {
  /** Generated CSS code */
  code: string;
  /** Source map (if requested) */
  map?: string;
  /** CSS variables used via v-bind() */
  cssVars: string[];
  /** Compilation errors */
  errors: string[];
  /** Compilation warnings */
  warnings: string[];
}

/** WASM Compiler class */
export declare class Compiler {
  constructor();

  /** Compile template to VDom render function */
  compile(template: string, options?: CompilerOptions): CompileResult;

  /** Compile template to Vapor mode */
  compileVapor(template: string, options?: CompilerOptions): CompileResult;

  /** Parse template to AST */
  parse(template: string, options?: CompilerOptions): object;

  /** Parse SFC (.vue file) */
  parseSfc(source: string, options?: SfcParseOptions): SfcDescriptor;

  /** Compile SFC (.vue file) */
  compileSfc(source: string, options?: SfcCompileOptions): SfcCompileResult;

  /** Compile CSS with LightningCSS */
  compileCss(css: string, options?: CssCompileOptions): CssCompileResult;

  /** Free the WASM memory (called automatically via Symbol.dispose) */
  free(): void;

  /** Disposable interface */
  [Symbol.dispose](): void;
}

/** Compile template to VDom render function */
export declare function compile(
  template: string,
  options?: CompilerOptions
): CompileResult;

/** Compile template to Vapor mode */
export declare function compileVapor(
  template: string,
  options?: CompilerOptions
): CompileResult;

/** Parse template to AST */
export declare function parseTemplate(
  template: string,
  options?: CompilerOptions
): object;

/** Parse SFC (.vue file) */
export declare function parseSfc(
  source: string,
  options?: SfcParseOptions
): SfcDescriptor;

/** Compile SFC (.vue file) */
export declare function compileSfc(
  source: string,
  options?: SfcCompileOptions
): SfcCompileResult;

/** Compile CSS with LightningCSS */
export declare function compileCss(
  css: string,
  options?: CssCompileOptions
): CssCompileResult;

/** Initialize the WASM module */
export declare function init(
  moduleOrPath?: RequestInfo | URL | Response | BufferSource | WebAssembly.Module
): Promise<void>;

/** Check if the WASM module is initialized */
export declare function isInitialized(): boolean;

export default init;
