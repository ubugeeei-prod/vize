/* eslint-disable */

/** CSS compile options for NAPI */
export interface CssCompileOptionsNapi {
  /** Filename for error reporting */
  filename?: string;
  /** Whether to apply scoped CSS transformation */
  scoped?: boolean;
  /** Scope ID for scoped CSS (e.g., "data-v-abc123"). Must be the full attribute name. */
  scopeId?: string;
  /** Whether to generate source maps */
  sourceMap?: boolean;
  /** Whether to minify the output */
  minify?: boolean;
  /** Whether to enable CSS Modules transforms */
  cssModules?: boolean;
  /** Whether to enable custom media query resolution */
  customMedia?: boolean;
  /** Browser targets for autoprefixing */
  targets?: CssTargetsNapi;
}

/** CSS compile result for NAPI */
export interface CssCompileResultNapi {
  /** Compiled CSS code */
  code: string;
  /** Source map (if requested) */
  map?: string;
  /** CSS variables found (from v-bind()) */
  cssVars: Array<string>;
  /** Errors during compilation */
  errors: Array<string>;
  /** Warnings during compilation */
  warnings: Array<string>;
}

/** Browser targets for CSS autoprefixing */
export interface CssTargetsNapi {
  chrome?: number;
  firefox?: number;
  safari?: number;
  edge?: number;
  ios?: number;
  android?: number;
}

/**
 * Compile a CSS string with scoped CSS, v-bind() extraction, and optional minification.
 * Unlike `compileSfc`, the `scopeId` is used as-is without stripping the "data-v-" prefix.
 * Callers must pass the full attribute name (e.g., "data-v-abc123").
 */
export declare function compileCss(
  source: string,
  options?: CssCompileOptionsNapi | undefined | null,
): CssCompileResultNapi;
