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
import { generateScopeId, prependInlineStyleInjection } from "./utils/index.ts";
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
  diagnostics?: { logWarnings?: boolean },
): CompiledModule {
  const content = source ?? fs.readFileSync(filePath, "utf-8");
  const resolved = resolveSfcSrcImports(filePath, content);
  const scopeId = generateScopeId(filePath);

  const result = compileSfc(resolved.source, buildCompileFileOptions(filePath, options));

  if (result.errors.length > 0) {
    throw new VizeSfcCompileError(filePath, result.errors);
  }

  if (result.warnings.length > 0 && (diagnostics?.logWarnings ?? true)) {
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
  /** Default JSX output mode; takes precedence over `vapor`. */
  jsxMode?: "vdom" | "vapor";
  vapor?: boolean;
  /**
   * SSR build: skip the runtime `<style>` injection for any extracted
   * `<style scoped>` CSS (it relies on `document`), mirroring the SFC inline-CSS
   * path which only injects on the client.
   */
  ssr?: boolean;
  /**
   * Request a v3 source map for the generated render code (#1533). When set,
   * the returned `map` carries the map JSON (or `null` when the native compiler
   * produced none, e.g. a Vapor component or a multi-component module).
   */
  sourceMap?: boolean;
}

/**
 * Compile a `.jsx`/`.tsx` Vue component module to render code through Vize.
 *
 * Mirrors `compileFile` for SFCs but routes through the native `compileJsx`
 * binding. The returned `code` is a self-contained module (the runtime-helper
 * preamble is included, #1533), and `map` carries a v3 source map when
 * `sourceMap` is requested. A component's `<style scoped>` CSS is surfaced
 * (already scope-rewritten) and emitted through the same inline-injection path
 * plain SFC `<style>` blocks use (#1495, #1533).
 */
export function compileJsxModule(
  filePath: string,
  source: string,
  options: JsxCompileFileOptions = {},
): { code: string; map: string | null; warnings: string[] } {
  const result = compileJsx(source, {
    filename: filePath,
    lang: filePath.endsWith(".tsx") ? "tsx" : "jsx",
    jsxMode: options.jsxMode,
    vapor: options.vapor ?? false,
    sourceMap: options.sourceMap ?? false,
  });

  if (result.errors.length > 0) {
    throw new VizeSfcCompileError(filePath, result.errors);
  }

  // Emit the extracted, scope-rewritten `<style scoped>` CSS through the shared
  // inline-injection path. The compiler already baked the `data-v-<hash>` scope
  // id into both the selectors and the render output, so the blocks are emitted
  // verbatim. Skipped under SSR, matching the SFC inline-CSS path.
  const css = (result.scopedStyles ?? []).map((style) => style.css).join("\n");
  let code = result.code;
  // Prepending the inline-style injection shifts the render code, so the v3 map
  // (which targets the unshifted render code) is dropped once we mutate `code`.
  let map = result.map ?? null;
  if (css && !options.ssr) {
    const styleKey = result.scopedStyles[0].scopeId.replace(/^data-v-/, "");
    code = prependInlineStyleInjection(code, css, styleKey);
    map = null;
  }

  return {
    code,
    map,
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
