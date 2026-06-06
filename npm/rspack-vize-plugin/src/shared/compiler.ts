/** Core SFC compilation logic. */

import { createHash } from "node:crypto";
import * as native from "@vizejs/native";
import type { CompiledModule, SfcCompileOptionsNapi } from "../types/index.ts";
import {
  generateScopeId,
  collectTemplateAssetUrls,
  toCustomBlockInfo,
  toStyleBlockInfo,
} from "./utils.ts";

export { generateOutput } from "./output.ts";

const { compileSfc } = native;

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
