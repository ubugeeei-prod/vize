/* eslint-disable */

/** Compile result */
export interface CompileResult {
  /** Generated code */
  code: string;
  /** Preamble code (imports) */
  preamble: string;
  /** AST (serialized as JSON) */
  ast: any;
  /** Source map */
  map?: any;
  /** Used helpers */
  helpers: Array<string>;
  /** Template strings for Vapor mode static parts */
  templates?: Array<string>;
}

/** Compiler options for bindings */
export interface CompilerOptions {
  /** Output mode: "module" or "function" */
  mode?: string;
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
  outputMode?: string;
  /** Whether the template contains TypeScript */
  isTs?: boolean;
  /** Whether the template targets a custom renderer instead of the DOM. */
  customRenderer?: boolean;
  /** Template syntax compatibility mode. */
  templateSyntax?: "standard" | "strict" | "quirks";
  /**
   * Script extension handling: "preserve" (keep TypeScript) or "downcompile" (transpile to JS)
   * Defaults to "downcompile"
   */
  scriptExt?: string;
}

/** Compile Vue template to VDom render function */
export declare function compile(
  template: string,
  options?: CompilerOptions | undefined | null,
): CompileResult;

/** Compile Vue template to Vapor mode */
export declare function compileVapor(
  template: string,
  options?: CompilerOptions | undefined | null,
): CompileResult;

/** Parse template to AST only */
export declare function parseTemplate(
  template: string,
  options?: CompilerOptions | undefined | null,
): any;
