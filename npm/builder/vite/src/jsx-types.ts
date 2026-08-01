/**
 * The native `compileJsx` binding surface.
 *
 * Split out of `types.ts` (which is over the repo's per-file line budget) so the
 * `.jsx`/`.tsx` boundary types live next to each other; `types.ts` re-exports
 * them, so every existing import site is unchanged.
 */

/** Options for the native `compileJsx` binding. */
export interface JsxCompileOptionsNapi {
  filename?: string;
  lang?: string;
  /**
   * Default JSX output mode (`"vdom"` | `"vapor"`). Mirrors `compiler.jsxMode`
   * and takes precedence over `vapor`. Per-component `"use vue:*"` directives
   * still override it.
   */
  jsxMode?: "vdom" | "vapor";
  vapor?: boolean;
  /**
   * Emit a v3 source map for the generated render code (#1533). When `true`,
   * the result's `map` carries the map JSON; `null` otherwise.
   */
  sourceMap?: boolean;
}

/** A JSX component's extracted `<style scoped>` block (#1495, #1533). */
export interface JsxScopedStyleNapi {
  /** Generated scope id, e.g. `data-v-1a2b3c4d`, already applied to the CSS. */
  scopeId: string;
  /** Scope-rewritten CSS, with the `data-v-<hash>` attribute applied. */
  css: string;
}

/** Result of the native `compileJsx` binding. */
export interface JsxCompileResultNapi {
  /**
   * Self-contained module: the deduplicated runtime-helper preamble followed by
   * every component's render code, in source order (the helper imports are no
   * longer dropped, #1533).
   */
  code: string;
  /**
   * v3 source map (JSON) for `code`, present only when `sourceMap` was
   * requested and the module is a single component. `null` otherwise (#1533).
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

export type CompileJsxFn = (
  source: string,
  options?: JsxCompileOptionsNapi,
) => JsxCompileResultNapi;
