/** Native compiler API types used by the Rspack adapter. */

// Native API Types

export interface SfcCompileOptionsNapi {
  filename?: string;
  sourceMap?: boolean;
  ssr?: boolean;
  vapor?: boolean;
  /** Template syntax compatibility mode */
  templateSyntax?: "standard" | "strict" | "quirks";
  experimentalInTagComments?: boolean;
  experimentalPatternedTemplate?: boolean;
  experimentalSelfComponent?: boolean;
  experimentalStrictSlotChildren?: boolean;
  experimentalServerScript?: boolean;
  /** Preserve TypeScript in output when true */
  isTs?: boolean;
  /** Scope ID for scoped CSS (e.g., "data-v-abc123") */
  scopeId?: string;
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

export interface SfcBlockAttributeNapi {
  name: string;
  value?: string;
}

export interface CustomBlockNapi {
  blockType: string;
  content: string;
  src?: string;
  attrs: SfcBlockAttributeNapi[];
  index: number;
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
  /** Source map v3 JSON when requested from the native compiler. */
  map?: string;
  errors: string[];
  warnings: string[];
  hasScoped: boolean;
  styles: StyleBlockNapi[];
  customBlocks: CustomBlockNapi[];
  macroArtifacts?: MacroArtifact[];
}

// JSX Compile API Types

/** Options for the native `compileJsx`. */
export interface JsxCompileOptionsNapi {
  /** Source filename, used to infer the language when `lang` is omitted. */
  filename?: string;
  /** Source language: "jsx" or "tsx". Inferred from a `.tsx` filename. */
  lang?: string;
  /**
   * Default output mode (`"vdom"` | `"vapor"`); mirrors `compiler.jsxMode` and
   * wins over `vapor`. Per-component `"use vue:*"` directives still override it.
   */
  jsxMode?: "vdom" | "vapor";
  /** JSX semantics; `"babel"` opts into @vue/babel-plugin-jsx compatibility. */
  jsxCompat?: "native" | "babel";
  /** Legacy default-mode toggle: `true` → Vapor, `false` (default) → VDOM. */
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

/** Result of the native `compileJsx`. */
export interface JsxCompileResultNapi {
  /**
   * Generated render code for the module: the deduplicated runtime-helper
   * preamble followed by every component's render code (the helper imports are
   * no longer dropped, #1533).
   */
  code: string;
  /**
   * v3 source map (JSON) for `code`, present only when `sourceMap` was requested
   * and the module is a single component. `null`/absent otherwise (#1533).
   */
  map?: string;
  /** Error-severity diagnostic messages. */
  errors: string[];
  /** Warning-severity diagnostic messages. */
  warnings: string[];
  /**
   * Extracted `<style scoped>` blocks across the module's components, in source
   * order (#1495). Empty when no component had a `<style scoped>`. Each entry's
   * CSS is already scope-rewritten; the plugin emits it through the same style
   * path SFC `<style>` blocks use (#1533).
   */
  scopedStyles: JsxScopedStyleNapi[];
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
