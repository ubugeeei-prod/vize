/** Core SFC compilation logic. */

import { createHash } from "node:crypto";
import * as native from "@vizejs/native";
import type {
  CompiledModule,
  JsxCompileResultNapi,
  SfcCompileOptionsNapi,
} from "../types/index.ts";
import {
  generateScopeId,
  collectTemplateAssetUrls,
  toCustomBlockInfo,
  toStyleBlockInfo,
} from "./utils.ts";

export { generateOutput } from "./output.ts";

const { compileSfc } = native;

const { compileJsx } = native as {
  compileJsx: (source: string, options?: Record<string, unknown>) => JsxCompileResultNapi;
};

/** `.jsx`/`.tsx` Vue components routed to the native JSX compiler. */
export function isJsxFile(filePath: string): boolean {
  return filePath.endsWith(".jsx") || filePath.endsWith(".tsx");
}

/**
 * Prepend a runtime `<style>` injection for plain CSS to a module's output.
 *
 * Mirrors the inline-CSS path the Vite/unplugin integrations use for plain SFC
 * `<style>` blocks: a guarded, idempotent `document.createElement("style")`
 * keyed by a stable id. The JSX compiler already scope-rewrites the CSS (the
 * `data-v-<hash>` attribute is baked into the selectors and the render output),
 * so the content is emitted verbatim (#1495, #1533).
 */
function prependInlineStyleInjection(output: string, css: string, styleKey: string): string {
  const cssCode = JSON.stringify(css);
  const cssId = JSON.stringify(`vize-style-${styleKey}`);
  return `
export const __vize_css__ = ${cssCode};
const __vize_css_id__ = ${cssId};
(function() {
  if (typeof document !== "undefined") {
    let style = document.getElementById(__vize_css_id__);
    if (!style) {
      style = document.createElement("style");
      style.id = __vize_css_id__;
      style.textContent = __vize_css__;
      document.head.appendChild(style);
    } else {
      style.textContent = __vize_css__;
    }
  }
})();
${output}`;
}

/**
 * Compile a `.jsx`/`.tsx` Vue module to render code via the native JSX
 * compiler. Mirrors {@link compileFile} but for the JSX lowering path: no custom
 * blocks or asset-url rewriting apply. A component's `<style scoped>` CSS is
 * surfaced (already scope-rewritten) and emitted through the same inline-style
 * injection path the integrations use for plain SFC CSS (#1495, #1533).
 */
export function compileJsxModule(
  filePath: string,
  source: string,
  options: { jsxMode?: "vdom" | "vapor"; vapor?: boolean; sourceMap?: boolean } = {},
): { code: string; map: string | null; warnings: string[] } {
  const result = compileJsx(source, {
    filename: filePath,
    lang: filePath.endsWith(".tsx") ? "tsx" : "jsx",
    jsxMode: options.jsxMode,
    vapor: options.vapor ?? false,
    sourceMap: options.sourceMap ?? false,
  });

  if (result.errors.length > 0) {
    throw new Error(`[vize] Compilation failed for ${filePath}:\n${result.errors.join("\n")}`);
  }

  const css = (result.scopedStyles ?? []).map((style) => style.css).join("\n");
  let code = result.code;
  // The native v3 map targets the unshifted render code; drop it once the
  // inline-style injection prepends to `code` (#1533).
  let map = result.map ?? null;
  if (css) {
    const styleKey = result.scopedStyles[0].scopeId.replace(/^data-v-/, "");
    code = prependInlineStyleInjection(code, css, styleKey);
    map = null;
  }

  return { code, map, warnings: result.warnings };
}

// Compilation Cache

interface CacheEntry {
  contentHash: string;
  result: CompiledModule;
}

/** Content-hash keyed cache for watch mode. */
const compilationCache = new Map<string, CacheEntry>();

function computeContentHash(source: string): string {
  return createHash("sha256").update(source).digest("hex").slice(0, 16);
}

/** Clear the compilation cache. Exposed for testing. */
export function clearCompilationCache(): void {
  compilationCache.clear();
}

/** Compile a .vue file with content-hash caching. */
export function compileFile(
  filePath: string,
  source: string,
  options: {
    sourceMap?: boolean;
    ssr?: boolean;
    vapor?: boolean;
    compilerOptions?: SfcCompileOptionsNapi;
    isCustomElement?: boolean;
    rootContext?: string;
    isProduction?: boolean;
    /** @see VizeLoaderOptions.transformAssetUrls */
    transformAssetUrls?: boolean | Record<string, string[]>;
  } = {},
): CompiledModule {
  // Auto-detect TypeScript
  const autoIsTs = options.compilerOptions?.isTs ?? /<script[^>]*\blang=["']ts["']/.test(source);

  // Composite cache key
  const ssr = options.ssr ?? options.compilerOptions?.ssr ?? false;
  const vapor = options.vapor ?? options.compilerOptions?.vapor ?? false;
  const sourceMap = options.sourceMap ?? options.compilerOptions?.sourceMap ?? true;
  const isCustomElement = options.isCustomElement ?? false;
  const rootCtx = options.rootContext ?? "";
  const isProd = options.isProduction ?? false;
  // Normalize transformAssetUrls for cache key
  const transformAssetUrls = options.transformAssetUrls ?? true;
  const templateSyntax = options.compilerOptions?.templateSyntax ?? "standard";
  const tauKey =
    transformAssetUrls === false
      ? "tau=false"
      : transformAssetUrls === true
        ? "tau=true"
        : `tau=${JSON.stringify(transformAssetUrls)}`;
  const cacheKey = `${filePath}:ssr=${ssr}:vapor=${vapor}:ts=${autoIsTs}:map=${sourceMap}:ce=${isCustomElement}:syntax=${templateSyntax}:root=${rootCtx}:prod=${isProd}:${tauKey}`;

  const contentHash = computeContentHash(source);
  const cached = compilationCache.get(cacheKey);
  if (cached && cached.contentHash === contentHash) {
    return cached.result;
  }

  const scopeId = generateScopeId(filePath, options.rootContext, options.isProduction, source);

  const napiOptions: SfcCompileOptionsNapi = {
    ...options.compilerOptions,
    filename: filePath,
    sourceMap: options.sourceMap ?? options.compilerOptions?.sourceMap ?? true,
    ssr,
    vapor,
    isTs: autoIsTs,
    scopeId: `data-v-${scopeId}`,
  };

  const result = compileSfc(source, napiOptions);

  const templateAssetUrls = collectTemplateAssetUrls(source, transformAssetUrls);

  const compiled: CompiledModule = {
    code: result.code,
    css: result.css,
    errors: result.errors,
    warnings: result.warnings,
    scopeId,
    hasScoped: result.hasScoped,
    styles: result.styles.map(toStyleBlockInfo),
    customBlocks: result.customBlocks.map(toCustomBlockInfo),
    isCustomElement,
    templateAssetUrls,
    macroArtifacts: result.macroArtifacts ?? [],
  };

  // Only cache successful compilations
  if (compiled.errors.length === 0) {
    compilationCache.set(cacheKey, { contentHash, result: compiled });
  }

  return compiled;
}
