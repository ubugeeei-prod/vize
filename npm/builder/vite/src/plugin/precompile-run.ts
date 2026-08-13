/**
 * The pre-compilation pass.
 *
 * Scans the configured patterns, diffs them against the previous in-process
 * run, restores whatever the persistent cache can prove is still valid, and
 * batch-compiles the rest. Extracted from `state.ts` so the state module stays
 * a state module.
 */

import fs from "node:fs";
import { glob } from "tinyglobby";

import { compileBatch, formatCompileErrorMessage } from "../compiler.ts";
import { buildCompileBatchOptions, type CompileBatchOptions } from "../compile-options.ts";
import {
  chunkPrecompileFiles,
  diffPrecompileFiles,
  isPrecompileSfcPath,
  type PrecompileFileMetadata,
} from "./precompile.ts";
import {
  createDisabledPrecompileCache,
  hashPrecompileSource,
  openPrecompileCache,
  type PrecompileCache,
} from "./precompile-cache.ts";
import {
  getCompileOptionsForRequest,
  syncCollectedCssForFile,
  type VizePluginState,
} from "./state.ts";
import { isPluginVueCustomElement } from "./plugin-vue-options.ts";

/**
 * The options the pre-compile batch actually compiles with.
 *
 * Also the input to the cache key, so the two can never drift: a new option
 * that reaches the native compiler reaches the key with it.
 */
function resolvePrecompileBatchOptions(state: VizePluginState): CompileBatchOptions {
  const requestOptions = getCompileOptionsForRequest(state, false);
  return {
    // The batch fills the same caches `load` serves from, so it has to make the
    // same source-map decision the on-demand path makes (#3399).
    sourceMap: requestOptions.sourceMap,
    ssr: false,
    vapor: state.mergedOptions.vapor ?? false,
    mode: state.mergedOptions.mode,
    customRenderer: state.mergedOptions.customRenderer ?? false,
    templateSyntax: state.mergedOptions.templateSyntax ?? "standard",
    experimentalInTagComments: state.mergedOptions.experimentalInTagComments ?? false,
    experimentalPatternedTemplate: state.mergedOptions.experimentalPatternedTemplate ?? false,
    experimentalServerScript: state.mergedOptions.experimentalServerScript ?? false,
    runtimeModuleName: state.mergedOptions.runtimeModuleName,
    runtimeGlobalName: state.mergedOptions.runtimeGlobalName,
    vueVersion: state.mergedOptions.vueVersion,
    styleTrim: requestOptions.styleTrim,
    templateCacheHandlers: requestOptions.templateCacheHandlers,
    templateComments: requestOptions.templateComments,
    templateHoistStatic: requestOptions.templateHoistStatic,
    templatePrefixIdentifiers: requestOptions.templatePrefixIdentifiers,
  };
}

function openCacheForRun(
  state: VizePluginState,
  batchOptions: CompileBatchOptions,
): PrecompileCache {
  if (!state.root) {
    return createDisabledPrecompileCache();
  }
  return openPrecompileCache({
    root: state.root,
    compileOptions: buildCompileBatchOptions(batchOptions),
    onDiagnostic: (message, error) =>
      error === undefined ? state.logger.warn(message) : state.logger.warn(message, error),
  });
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
  const sfcFiles = files.filter(isPrecompileSfcPath);

  const currentMetadata = new Map<string, PrecompileFileMetadata>();
  for (const file of sfcFiles) {
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
    sfcFiles,
    currentMetadata,
    state.precompileMetadata,
  );
  const cachedFileCount = sfcFiles.length - changedFiles.length;

  for (const file of deletedFiles) {
    state.cache.delete(file);
    state.ssrCache.delete(file);
    if (state.extractCss) {
      state.collectedCss.delete(file);
    }
    state.precompileMetadata.delete(file);
    state.pendingHmrUpdateTypes.delete(file);
  }

  state.logger.info(
    `Pre-compiling ${sfcFiles.length} Vue files... (${changedFiles.length} changed, ${cachedFileCount} cached, ${deletedFiles.length} removed)`,
  );

  if (changedFiles.length === 0) {
    const elapsed = (performance.now() - startTime).toFixed(2);
    state.logger.info(`Pre-compilation complete: cache reused (${elapsed}ms)`);
    return;
  }

  for (const file of changedFiles) {
    if (state.extractCss) {
      state.collectedCss.delete(file);
    }
    state.pendingHmrUpdateTypes.delete(file);
  }

  const batchOptions = resolvePrecompileBatchOptions(state);
  const cache = openCacheForRun(state, batchOptions);
  // A disabled cache must not pay for hashing it will never consult.
  const cacheEnabled = cache.file !== null;

  let successCount = 0;
  let restoredCount = 0;
  let failedCount = 0;
  let nativeTimeMs = 0;
  const precompileFailures: string[] = [];
  const chunks = chunkPrecompileFiles(changedFiles, state.precompileBatchSize, {
    metadata: currentMetadata,
  });

  for (const chunk of chunks) {
    const fileContents: { path: string; source: string }[] = [];
    const sourceHashes = new Map<string, string>();
    for (const file of chunk) {
      let source: string;
      try {
        source = fs.readFileSync(file, "utf-8");
      } catch (e) {
        failedCount++;
        state.cache.delete(file);
        if (state.extractCss) {
          state.collectedCss.delete(file);
        }
        state.precompileMetadata.delete(file);
        cache.delete(file);
        precompileFailures.push(`[vize] Failed to read ${file}: ${formatUnknownError(e)}`);
        state.logger.error(`Failed to read ${file}:`, e);
        continue;
      }

      // The hash is taken from the bytes just read, so a cache hit means the
      // stored output was produced from exactly this source text.
      const sourceHash = cacheEnabled ? hashPrecompileSource(source) : undefined;
      const restored = sourceHash === undefined ? undefined : cache.get(file, sourceHash);
      if (restored) {
        state.cache.set(file, restored);
        const metadata = currentMetadata.get(file);
        if (metadata) {
          state.precompileMetadata.set(file, metadata);
        }
        syncCollectedCssForFile(
          {
            ...state,
            extractCss: state.extractCss && !isPluginVueCustomElement(state.mergedOptions, file),
          },
          file,
          restored,
        );
        restoredCount++;
        continue;
      }

      if (sourceHash !== undefined) {
        sourceHashes.set(file, sourceHash);
      }
      fileContents.push({ path: file, source });
    }

    if (fileContents.length === 0) {
      continue;
    }

    const result = compileBatch(fileContents, state.cache, batchOptions);

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
        if (state.extractCss) {
          state.collectedCss.delete(fileResult.path);
        }
        state.precompileMetadata.delete(fileResult.path);
        cache.delete(fileResult.path);
        precompileFailures.push(formatCompileErrorMessage(fileResult.path, fileResult.errors));
        continue;
      }

      if (metadata) {
        state.precompileMetadata.set(fileResult.path, metadata);
      }

      const compiled = state.cache.get(fileResult.path);
      const sourceHash = sourceHashes.get(fileResult.path);
      if (compiled && sourceHash !== undefined) {
        cache.set(fileResult.path, sourceHash, compiled);
      }

      syncCollectedCssForFile(
        {
          ...state,
          extractCss:
            state.extractCss && !isPluginVueCustomElement(state.mergedOptions, fileResult.path),
        },
        fileResult.path,
        compiled,
      );
    }
  }

  // Files that vanished from the scan must not accumulate in the manifest.
  cache.retain(new Set(sfcFiles));
  cache.flush();

  const elapsed = (performance.now() - startTime).toFixed(2);
  const batchLabel = chunks.length === 1 ? "batch" : "batches";
  state.logger.info(
    `Pre-compilation complete: ${successCount} recompiled, ${restoredCount} restored from disk, ${cachedFileCount} reused, ${failedCount} failed (${elapsed}ms, native ${batchLabel}: ${nativeTimeMs.toFixed(2)}ms)`,
  );

  if (failedCount > 0) {
    const details = precompileFailures.length > 0 ? `\n\n${precompileFailures.join("\n\n")}` : "";
    throw new Error(`[vize] Pre-compilation failed for ${failedCount} file(s).${details}`);
  }
}

function formatUnknownError(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}
