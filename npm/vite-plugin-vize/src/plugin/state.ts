/**
 * Plugin state type and batch compilation logic.
 */

import type { ViteDevServer } from "vite";
import fs from "node:fs";
import { glob } from "tinyglobby";

import type { VizeOptions, CompiledModule } from "../types.ts";
import { compileBatch, formatCompileErrorMessage } from "../compiler.ts";
import { resolveCssImports, type CssAliasRule } from "../utils/css.ts";
import { hasDelegatedStyles } from "../utils/index.ts";
import { type DynamicImportAliasRule } from "../virtual.ts";
import { createLogger } from "../transform.ts";
import type { HmrUpdateType } from "../hmr.ts";
import {
  chunkPrecompileFiles,
  diffPrecompileFiles,
  type PrecompileFileMetadata,
} from "./precompile.ts";

export {
  DEFAULT_PRECOMPILE_BATCH_MAX_BYTES,
  DEFAULT_PRECOMPILE_BATCH_SIZE,
  DEFAULT_PRECOMPILE_IGNORE_PATTERNS,
  chunkPrecompileFiles,
  diffPrecompileFiles,
  hasFileMetadataChanged,
  normalizePrecompileBatchSize,
  type PrecompileChunkOptions,
  type PrecompileDiff,
  type PrecompileFileMetadata,
} from "./precompile.ts";

export interface VizePluginState {
  cache: Map<string, CompiledModule>;
  ssrCache: Map<string, CompiledModule>;
  collectedCss: Map<string, string>;
  precompileMetadata: Map<string, PrecompileFileMetadata>;
  pendingHmrUpdateTypes: Map<string, HmrUpdateType>;
  isProduction: boolean;
  root: string;
  clientViteBase: string;
  serverViteBase: string;
  server: ViteDevServer | null;
  filter: (id: string) => boolean;
  scanPatterns: string[] | null;
  precompileBatchSize: number;
  ignorePatterns: string[];
  mergedOptions: VizeOptions;
  initialized: boolean;
  dynamicImportAliasRules: DynamicImportAliasRule[];
  cssAliasRules: CssAliasRule[];
  extractCss: boolean;
  clientViteDefine: Record<string, string>;
  serverViteDefine: Record<string, string>;
  logger: ReturnType<typeof createLogger>;
}

export function getEnvironmentCache(
  state: Pick<VizePluginState, "cache" | "ssrCache">,
  ssr: boolean,
): Map<string, CompiledModule> {
  return ssr ? state.ssrCache : state.cache;
}

export function getCompileOptionsForRequest(
  state: Pick<VizePluginState, "isProduction" | "mergedOptions">,
  ssr: boolean,
): {
  sourceMap: boolean;
  ssr: boolean;
  vapor: boolean;
  customRenderer: boolean;
  vueParserQuirks: boolean;
} {
  return {
    sourceMap: state.mergedOptions?.sourceMap ?? !state.isProduction,
    ssr,
    // Vapor runtime is client-oriented today; use VDOM for SSR and Vapor on the client.
    vapor: !ssr && (state.mergedOptions?.vapor ?? false),
    customRenderer: state.mergedOptions?.customRenderer ?? false,
    vueParserQuirks: state.mergedOptions?.vueParserQuirks ?? false,
  };
}

export function syncCollectedCssForFile(
  state: Pick<VizePluginState, "isProduction" | "collectedCss" | "cssAliasRules">,
  filePath: string,
  compiled: CompiledModule | undefined,
): void {
  if (!compiled || !state.isProduction) {
    return;
  }

  if (compiled.css && !hasDelegatedStyles(compiled)) {
    state.collectedCss.set(
      filePath,
      resolveCssImports(compiled.css, filePath, state.cssAliasRules, false),
    );
  } else {
    state.collectedCss.delete(filePath);
  }
}

export function clearBuildCaches(
  state: Pick<
    VizePluginState,
    "cache" | "collectedCss" | "pendingHmrUpdateTypes" | "precompileMetadata" | "ssrCache"
  >,
): void {
  state.cache.clear();
  state.ssrCache.clear();
  state.collectedCss.clear();
  state.precompileMetadata.clear();
  state.pendingHmrUpdateTypes.clear();
}

/**
 * Pre-compile all Vue files matching scan patterns.
 */
export async function compileAll(state: VizePluginState): Promise<void> {
  const startTime = performance.now();
  const files = await glob(state.scanPatterns!, {
    cwd: state.root,
    ignore: state.ignorePatterns,
    absolute: true,
  });

  const currentMetadata = new Map<string, PrecompileFileMetadata>();
  for (const file of files) {
    try {
      const stat = fs.statSync(file);
      currentMetadata.set(file, {
        mtimeMs: stat.mtimeMs,
        size: stat.size,
      });
    } catch (e) {
      state.logger.error(`Failed to stat ${file}:`, e);
    }
  }

  const { changedFiles, deletedFiles } = diffPrecompileFiles(
    files,
    currentMetadata,
    state.precompileMetadata,
  );
  const cachedFileCount = files.length - changedFiles.length;

  for (const file of deletedFiles) {
    state.cache.delete(file);
    state.ssrCache.delete(file);
    state.collectedCss.delete(file);
    state.precompileMetadata.delete(file);
    state.pendingHmrUpdateTypes.delete(file);
  }

  state.logger.info(
    `Pre-compiling ${files.length} Vue files... (${changedFiles.length} changed, ${cachedFileCount} cached, ${deletedFiles.length} removed)`,
  );

  if (changedFiles.length === 0) {
    const elapsed = (performance.now() - startTime).toFixed(2);
    state.logger.info(`Pre-compilation complete: cache reused (${elapsed}ms)`);
    return;
  }

  for (const file of changedFiles) {
    state.collectedCss.delete(file);
    state.pendingHmrUpdateTypes.delete(file);
  }

  let successCount = 0;
  let failedCount = 0;
  let nativeTimeMs = 0;
  const precompileFailures: string[] = [];
  const chunks = chunkPrecompileFiles(changedFiles, state.precompileBatchSize, {
    metadata: currentMetadata,
  });

  for (const chunk of chunks) {
    const fileContents: { path: string; source: string }[] = [];
    for (const file of chunk) {
      try {
        const source = fs.readFileSync(file, "utf-8");
        fileContents.push({ path: file, source });
      } catch (e) {
        failedCount++;
        state.cache.delete(file);
        state.collectedCss.delete(file);
        state.precompileMetadata.delete(file);
        precompileFailures.push(`[vize] Failed to read ${file}: ${formatUnknownError(e)}`);
        state.logger.error(`Failed to read ${file}:`, e);
      }
    }

    if (fileContents.length === 0) {
      continue;
    }

    const result = compileBatch(fileContents, state.cache, {
      ssr: false,
      vapor: state.mergedOptions.vapor ?? false,
      customRenderer: state.mergedOptions.customRenderer ?? false,
      vueParserQuirks: state.mergedOptions.vueParserQuirks ?? false,
    });

    const chunkFailedCount = result.results.filter(
      (fileResult) => fileResult.errors.length > 0,
    ).length;
    failedCount += chunkFailedCount;
    successCount += result.results.length - chunkFailedCount;
    nativeTimeMs += result.timeMs;

    // Collect CSS for production extraction.
    // Skip files with delegated styles (preprocessor/CSS Modules) -- those go through
    // Vite's CSS pipeline and are extracted by Vite itself.
    for (const fileResult of result.results) {
      const metadata = currentMetadata.get(fileResult.path);

      if (fileResult.errors.length > 0) {
        state.cache.delete(fileResult.path);
        state.collectedCss.delete(fileResult.path);
        state.precompileMetadata.delete(fileResult.path);
        precompileFailures.push(formatCompileErrorMessage(fileResult.path, fileResult.errors));
        continue;
      }

      if (metadata) {
        state.precompileMetadata.set(fileResult.path, metadata);
      }

      syncCollectedCssForFile(state, fileResult.path, state.cache.get(fileResult.path));
    }
  }

  const elapsed = (performance.now() - startTime).toFixed(2);
  const batchLabel = chunks.length === 1 ? "batch" : "batches";
  state.logger.info(
    `Pre-compilation complete: ${successCount} recompiled, ${cachedFileCount} reused, ${failedCount} failed (${elapsed}ms, native ${batchLabel}: ${nativeTimeMs.toFixed(2)}ms)`,
  );

  if (failedCount > 0) {
    const details = precompileFailures.length > 0 ? `\n\n${precompileFailures.join("\n\n")}` : "";
    throw new Error(`[vize] Pre-compilation failed for ${failedCount} file(s).${details}`);
  }
}

function formatUnknownError(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}
