import fs from "node:fs";
import path from "node:path";
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
import type { CompileJsxFn } from "./types.ts";

const { compileSfc, compileSfcBatchWithResults } = native;
const compileJsx = (native as { compileJsx: CompileJsxFn }).compileJsx;

export class VizeSfcCompileError extends Error {
  readonly filePath: string;
  readonly diagnostics: readonly string[];

  constructor(filePath: string, diagnostics: readonly string[]) {
    super(formatCompileErrorMessage(filePath, diagnostics));
    this.name = "VizeSfcCompileError";
    this.filePath = filePath;
    this.diagnostics = diagnostics;
  }
}

export function formatCompileErrorMessage(
  filePath: string,
  diagnostics: readonly string[],
): string {
  const details = diagnostics.map((diagnostic) => `  - ${diagnostic}`).join("\n");
  return `[vize] Compilation failed in ${filePath}:\n${details}`;
}

function normalizeStyleBlocks(styles: NativeStyleBlockInfo[] | undefined): StyleBlockInfo[] {
  if (!styles) {
    return [];
  }

  return styles.map((block) => ({
    content: block.content,
    src: block.src ?? null,
    lang: block.lang ?? null,
    scoped: block.scoped,
    module: block.module ? (block.moduleName ?? true) : false,
    index: block.index,
  }));
}

export interface ResolvedSfcSrcImports {
  source: string;
  dependencies: string[];
}

function resolveRelativeSrc(filePath: string, src: string): string {
  return path.isAbsolute(src) ? src : path.resolve(path.dirname(filePath), src);
}

function readSrcImport(
  filePath: string,
  tag: string,
  src: string,
): { path: string; content: string } {
  const resolvedPath = resolveRelativeSrc(filePath, src);
  try {
    return {
      path: resolvedPath,
      content: fs.readFileSync(resolvedPath, "utf-8"),
    };
  } catch {
    throw new Error(
      `[vize] <${tag} src="${src}"> not found (resolved: ${resolvedPath}) in ${filePath}`,
    );
  }
}

function stripSrcAttribute(attrs: string): string {
  return attrs.replace(/\s*\bsrc\s*=\s*(?:"[^"]*"|'[^']*')/i, "");
}

function inlineSingleSrcBlock(
  source: string,
  filePath: string,
  tag: "script" | "template",
  src: string | undefined,
  dependencies: string[],
): string {
  if (!src) {
    return source;
  }

  const imported = readSrcImport(filePath, tag, src);
  dependencies.push(imported.path);
  const pattern = new RegExp(
    `<${tag}\\b([^>]*)\\bsrc\\s*=\\s*(['"])[^'"]+\\2([^>]*)>[\\s\\S]*?<\\/${tag}>`,
    "i",
  );

  return source.replace(pattern, (_match, beforeSrc: string, _quote: string, afterSrc: string) => {
    const attrs = stripSrcAttribute(`${beforeSrc}${afterSrc}`);
    return `<${tag}${attrs}>\n${imported.content}\n</${tag}>`;
  });
}

function inlineStyleSrcBlocks(source: string, filePath: string, dependencies: string[]): string {
  const pattern = /<style\b([^>]*)\bsrc\s*=\s*(['"])([^'"]+)\2([^>]*)>[\s\S]*?<\/style>/gi;

  return source.replace(
    pattern,
    (_match, beforeSrc: string, _quote: string, src: string, afterSrc: string) => {
      const imported = readSrcImport(filePath, "style", src);
      dependencies.push(imported.path);
      const attrs = stripSrcAttribute(`${beforeSrc}${afterSrc}`);
      return `<style${attrs}>\n${imported.content}\n</style>`;
    },
  );
}

export function resolveSfcSrcImports(filePath: string, source: string): ResolvedSfcSrcImports {
  const dependencies: string[] = [];
  const srcInfo = native.extractSfcSrcInfo(source, filePath);
  let resolvedSource = source;

  resolvedSource = inlineSingleSrcBlock(
    resolvedSource,
    filePath,
    "script",
    srcInfo.scriptSrc,
    dependencies,
  );
  resolvedSource = inlineSingleSrcBlock(
    resolvedSource,
    filePath,
    "template",
    srcInfo.templateSrc,
    dependencies,
  );
  resolvedSource = inlineStyleSrcBlocks(resolvedSource, filePath, dependencies);

  return {
    source: resolvedSource,
    dependencies,
  };
}

export function compileFile(
  filePath: string,
  cache: Map<string, CompiledModule>,
  options: CompileFileOptions,
  source?: string,
): CompiledModule {
  const content = source ?? fs.readFileSync(filePath, "utf-8");
  const resolved = resolveSfcSrcImports(filePath, content);
  const scopeId = generateScopeId(filePath);

  const result = compileSfc(resolved.source, buildCompileFileOptions(filePath, options));

  if (result.errors.length > 0) {
    throw new VizeSfcCompileError(filePath, result.errors);
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
    dependencies: resolved.dependencies,
  };

  cache.set(filePath, compiled);
  return compiled;
}

export interface JsxCompileFileOptions {
  vapor?: boolean;
}

/**
 * Compile a `.jsx`/`.tsx` Vue component module to render code through Vize.
 *
 * Mirrors `compileFile` for SFCs but routes through the native `compileJsx`
 * binding. JSX/TSX modules carry no `<style>` blocks or src imports, so the
 * result is just the generated render code plus any warnings.
 */
export function compileJsxModule(
  filePath: string,
  source: string,
  options: JsxCompileFileOptions = {},
): { code: string; warnings: string[] } {
  const result = compileJsx(source, {
    filename: filePath,
    lang: filePath.endsWith(".tsx") ? "tsx" : "jsx",
    vapor: options.vapor ?? false,
  });

  if (result.errors.length > 0) {
    throw new VizeSfcCompileError(filePath, result.errors);
  }

  return {
    code: result.code,
    warnings: result.warnings,
  };
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
  const dependenciesByPath = new Map<string, string[]>();
  const resolvedFiles = files.map((file) => {
    const resolved = resolveSfcSrcImports(file.path, file.source);
    dependenciesByPath.set(file.path, resolved.dependencies);
    return {
      path: file.path,
      source: resolved.source,
    };
  });

  const result = compileSfcBatchWithResults(
    resolvedFiles satisfies BatchFileInput[],
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
        dependencies: dependenciesByPath.get(fileResult.path) ?? [],
      });
    }

    // Log errors and warnings
    if (fileResult.errors.length > 0) {
      console.error(formatCompileErrorMessage(fileResult.path, fileResult.errors));
    }
    if (fileResult.warnings.length > 0) {
      fileResult.warnings.forEach((warning) => {
        console.warn(`[vize] Warning in ${fileResult.path}: ${warning}`);
      });
    }
  }

  return result;
}
