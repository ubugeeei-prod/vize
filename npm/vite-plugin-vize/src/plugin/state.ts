/**
 * Plugin state type and batch compilation logic.
 */

import type { ViteDevServer } from "vite";
import fs from "node:fs";
import { glob } from "tinyglobby";

import type { VizeOptions, CompiledModule } from "../types.js";
import { compileBatch } from "../compiler.js";
import { resolveCssImports, type CssAliasRule } from "../utils/css.js";
import { hasDelegatedStyles } from "../utils/index.js";
import { type DynamicImportAliasRule } from "../virtual.js";
import { createLogger } from "../transform.js";
import type { HmrUpdateType } from "../hmr.js";

export interface PrecompileFileMetadata {
  mtimeMs: number;
  size: number;
}

export interface PrecompileDiff {
  changedFiles: string[];
  deletedFiles: string[];
}

export function hasFileMetadataChanged(
  previous: PrecompileFileMetadata | undefined,
  next: PrecompileFileMetadata,
): boolean {
  return previous === undefined || previous.mtimeMs !== next.mtimeMs || previous.size !== next.size;
}

export function diffPrecompileFiles(
  files: readonly string[],
  currentMetadata: ReadonlyMap<string, PrecompileFileMetadata>,
  previousMetadata: ReadonlyMap<string, PrecompileFileMetadata>,
): PrecompileDiff {
  const changedFiles: string[] = [];
  const seenFiles = new Set(files);

  for (const file of files) {
    const metadata = currentMetadata.get(file);
    if (!metadata || hasFileMetadataChanged(previousMetadata.get(file), metadata)) {
      changedFiles.push(file);
    }
  }

  const deletedFiles: string[] = [];
  for (const file of previousMetadata.keys()) {
    if (!seenFiles.has(file)) {
      deletedFiles.push(file);
    }
  }

  return { changedFiles, deletedFiles };
}

export interface VizePluginState {
  cache: Map<string, CompiledModule>;
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

  // Read all files
  const fileContents: { path: string; source: string }[] = [];
  for (const file of changedFiles) {
    try {
      const source = fs.readFileSync(file, "utf-8");
      fileContents.push({ path: file, source });
    } catch (e) {
      state.logger.error(`Failed to read ${file}:`, e);
    }
  }

  // Batch compile using native parallel processing
  const result = compileBatch(fileContents, state.cache, {
    ssr: state.mergedOptions.ssr ?? false,
  });

  for (const file of changedFiles) {
    state.collectedCss.delete(file);
    state.pendingHmrUpdateTypes.delete(file);
  }

  // Collect CSS for production extraction.
  // Skip files with delegated styles (preprocessor/CSS Modules) -- those go through
  // Vite's CSS pipeline and are extracted by Vite itself.
  for (const fileResult of result.results) {
    const metadata = currentMetadata.get(fileResult.path);

    if (fileResult.errors.length > 0) {
      state.cache.delete(fileResult.path);
      state.collectedCss.delete(fileResult.path);
      state.precompileMetadata.delete(fileResult.path);
      continue;
    }

    if (metadata) {
      state.precompileMetadata.set(fileResult.path, metadata);
    }

    if (state.isProduction && fileResult.css) {
      const cached = state.cache.get(fileResult.path);
      if (cached && !hasDelegatedStyles(cached)) {
        state.collectedCss.set(
          fileResult.path,
          resolveCssImports(fileResult.css, fileResult.path, state.cssAliasRules, false),
        );
      }
    }
  }

  const elapsed = (performance.now() - startTime).toFixed(2);
  state.logger.info(
    `Pre-compilation complete: ${result.successCount} recompiled, ${cachedFileCount} reused, ${result.failedCount} failed (${elapsed}ms, native batch: ${result.timeMs.toFixed(2)}ms)`,
  );
}
