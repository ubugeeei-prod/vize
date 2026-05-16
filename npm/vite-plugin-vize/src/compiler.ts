import fs from "node:fs";
import * as native from "@vizejs/native";
import type {
  CompiledModule,
  BatchFileInput,
  BatchCompileResultWithFiles,
  NativeStyleBlockInfo,
  StyleBlockInfo,
} from "./types.ts";
import {
  buildCompileBatchOptions,
  buildCompileFileOptions,
  type CompileBatchOptions,
  type CompileFileOptions,
} from "./compile-options.ts";
import { generateScopeId } from "./utils/index.ts";

const { compileSfc, compileSfcBatchWithResults } = native;

function normalizeStyleBlocks(styles: NativeStyleBlockInfo[] | undefined): StyleBlockInfo[] {
  if (!styles) {
    return [];
  }

  return styles.map((block) => ({
    content: block.content,
    lang: block.lang ?? null,
    scoped: block.scoped,
    module: block.module ? (block.moduleName ?? true) : false,
    index: block.index,
  }));
}

export function compileFile(
  filePath: string,
  cache: Map<string, CompiledModule>,
  options: CompileFileOptions,
  source?: string,
): CompiledModule {
  const content = source ?? fs.readFileSync(filePath, "utf-8");
  const scopeId = generateScopeId(filePath);

  const result = compileSfc(content, buildCompileFileOptions(filePath, options));

  if (result.errors.length > 0) {
    const errorMsg = result.errors.join("\n");
    console.error(`[vize] Compilation error in ${filePath}:\n${errorMsg}`);
  }

  if (result.warnings.length > 0) {
    result.warnings.forEach((warning) => {
      console.warn(`[vize] Warning in ${filePath}: ${warning}`);
    });
  }

  const compiled: CompiledModule = {
    code: result.code,
    css: result.css,
    scopeId,
    hasScoped: result.hasScoped,
    templateHash: result.templateHash,
    styleHash: result.styleHash,
    scriptHash: result.scriptHash,
    macroArtifacts: result.macroArtifacts ?? [],
    styles: normalizeStyleBlocks(result.styles),
  };

  cache.set(filePath, compiled);
  return compiled;
}

/**
 * Batch compile multiple files in parallel using native Rust multithreading.
 * Returns per-file results with content hashes for HMR.
 */
export function compileBatch(
  files: { path: string; source: string }[],
  cache: Map<string, CompiledModule>,
  options: CompileBatchOptions,
): BatchCompileResultWithFiles {
  const result = compileSfcBatchWithResults(
    files satisfies BatchFileInput[],
    buildCompileBatchOptions(options),
  );

  // Update cache with results
  for (const fileResult of result.results) {
    if (fileResult.errors.length === 0) {
      cache.set(fileResult.path, {
        code: fileResult.code,
        css: fileResult.css,
        scopeId: fileResult.scopeId,
        hasScoped: fileResult.hasScoped,
        templateHash: fileResult.templateHash,
        styleHash: fileResult.styleHash,
        scriptHash: fileResult.scriptHash,
        macroArtifacts: fileResult.macroArtifacts ?? [],
        styles: normalizeStyleBlocks(fileResult.styles),
      });
    }

    // Log errors and warnings
    if (fileResult.errors.length > 0) {
      console.error(
        `[vize] Compilation error in ${fileResult.path}:\n${fileResult.errors.join("\n")}`,
      );
    }
    if (fileResult.warnings.length > 0) {
      fileResult.warnings.forEach((warning) => {
        console.warn(`[vize] Warning in ${fileResult.path}: ${warning}`);
      });
    }
  }

  return result;
}
