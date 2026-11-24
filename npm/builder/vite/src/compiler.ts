import fs from "node:fs";
import * as native from "@vizejs/native";
import { prependMappedJsxCode } from "../../shared/source-map.ts";
import {
  assertSfcStyleSrcImports,
  inlineSfcSrcBlocks,
  type InlinedSfcSource,
} from "../../shared/sfc-src.ts";
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
import { normalizeCustomBlocks } from "./custom-blocks.ts";
import { generateScopeId, prependInlineStyleInjection } from "./utils/index.ts";
import type { CompileJsxFn } from "./types.ts";

const { compileSfc, compileSfcBatchWithResults } = native;
const compileJsx = (native as { compileJsx: CompileJsxFn }).compileJsx;

const registeredPluginPlans = new WeakMap<object, string>();

/**
 * Register the `vue.plugins` plan a resolved config projects (#13074) with the
 * native compiler once, and return the digest compile requests name it by.
 * The plan is re-verified natively; a config without plugins has none.
 */
export function registerPluginPlan(config: unknown): string | undefined {
  const projected =
    config !== null && typeof config === "object"
      ? (config as { __vizePluginPlan?: unknown }).__vizePluginPlan
      : undefined;
  if (projected === null || typeof projected !== "object") return undefined;
  const known = registeredPluginPlans.get(projected);
  if (known !== undefined) return known;
  const register = (native as { registerPluginPlan?: (projected: string) => string })
    .registerPluginPlan;
  if (typeof register !== "function") {
    throw new Error(
      "[plugin/host-unsupported] @vizejs/native: this native binding cannot host vue.plugins; upgrade @vizejs/native",
    );
  }
  const digest = register(JSON.stringify(projected));
  registeredPluginPlans.set(projected, digest);
  return digest;
}

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

function customBlockFields(
  blocks: Parameters<typeof normalizeCustomBlocks>[0],
): { customBlocks: ReturnType<typeof normalizeCustomBlocks> } | Record<string, never> {
  const customBlocks = normalizeCustomBlocks(blocks);
  return customBlocks.length > 0 ? { customBlocks } : {};
}

function normalizeStyleBlocks(
  styles: NativeStyleBlockInfo[] | undefined,
  inlined: InlinedSfcSource | undefined,
): StyleBlockInfo[] {
  if (!styles) {
    return [];
  }

  // An inlined `<style src>` block keeps the file it came from (#11522), and an
  // external one its request, for the output to import (#11494).
  return styles.map((block) => ({
    content: block.content,
    src: inlined?.styleSrcs.get(block.index) ?? block.src ?? null,
    ...(block.src != null && inlined?.imports.includes(block.src) ? { external: true } : {}),
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
  diagnostics?: { logWarnings?: boolean },
): CompiledModule {
  const content = source ?? fs.readFileSync(filePath, "utf-8");
  const resolved = inlineSfcSrcBlocks(
    native.parseSfc,
    filePath,
    content,
    options.srcResolutions?.get(filePath),
  );
  const scopeId = generateScopeId(filePath, options.root, options.isProd ?? false, content);

  const result = compileSfc(resolved.source, buildCompileFileOptions(filePath, options, scopeId));

  if (result.errors.length > 0) {
    throw new VizeSfcCompileError(filePath, result.errors);
  }
  assertSfcStyleSrcImports(filePath, result.styles ?? [], resolved.imports);

  if (result.warnings.length > 0 && (diagnostics?.logWarnings ?? true)) {
    result.warnings.forEach((warning) => {
      console.warn(`[vize] Warning in ${filePath}: ${warning}`);
    });
  }

  const compiled: CompiledModule = {
    ...(result.typeProject ? { typeProject: result.typeProject } : {}),
    source: content,
    code: result.code,
    // Set only when present: the persistent cache round-trips through JSON, so
    // an explicit `map: undefined` would not survive and a restored entry would
    // stop being byte-identical to the compiled one.
    ...(result.map ? { map: result.map } : {}),
    css: result.css,
    scopeId,
    hasScoped: result.hasScoped,
    templateHash: result.templateHash,
    styleHash: result.styleHash,
    scriptHash: result.scriptHash,
    macroArtifacts: result.macroArtifacts ?? [],
    styles: normalizeStyleBlocks(result.styles, resolved),
    ...customBlockFields(result.customBlocks),
    // `src` blocks, then what the compile itself read (pug includes).
    dependencies: [...resolved.dependencies, ...(result.dependencies ?? [])],
    // The emitted module's shape, so its output is never parsed again (#3425).
    ...(result.moduleShape ? { moduleShape: result.moduleShape } : {}),
  };

  cache.set(filePath, compiled);
  return compiled;
}

export interface JsxCompileFileOptions {
  /** Default JSX output mode; takes precedence over `vapor`. */
  jsxMode?: "vdom" | "vapor";
  /** JSX semantics; `"babel"` opts into @vue/babel-plugin-jsx compatibility. */
  jsxCompat?: "native" | "babel";
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
    jsxCompat: options.jsxCompat,
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
  let map = result.map ?? null;
  if (css && !options.ssr) {
    const styleKey = result.scopedStyles[0].scopeId.replace(/^data-v-/, "");
    ({ code, map } = prependMappedJsxCode(
      code,
      map,
      prependInlineStyleInjection("", css, styleKey),
    ));
  }

  return {
    code,
    map,
    warnings: result.warnings,
  };
}

export interface CompileBatchReporting {
  /**
   * Receives a failure that belongs to one file alone (a `src` block that
   * cannot be read or inlined) instead of that failure aborting the batch. The
   * file is then left out of the cache and of the diagnostics below.
   */
  onFileFailure?: (path: string, error: unknown) => void;
  /** Print each file's compiler errors and warnings. @default true */
  log?: boolean;
}

/**
 * Batch compile multiple files in parallel using native Rust multithreading.
 * Returns per-file results with content hashes for HMR.
 */
export function compileBatch(
  files: { path: string; source: string }[],
  cache: Map<string, CompiledModule>,
  options: CompileBatchOptions,
  reporting: CompileBatchReporting = {},
): BatchCompileResultWithFiles {
  const { onFileFailure, log = true } = reporting;
  const failed = new Set<string>();
  const isolate = (path: string, run: () => void): void => {
    if (!onFileFailure) return run();
    try {
      run();
    } catch (error) {
      failed.add(path);
      onFileFailure(path, error);
    }
  };
  const inlinedByPath = new Map<string, InlinedSfcSource>();
  const resolvedFiles: BatchFileInput[] = [];
  for (const file of files) {
    isolate(file.path, () => {
      const resolved = inlineSfcSrcBlocks(
        native.parseSfc,
        file.path,
        file.source,
        options.srcResolutions?.get(file.path),
      );
      inlinedByPath.set(file.path, resolved);
      resolvedFiles.push({
        path: file.path,
        source: resolved.source,
        scopeId: generateScopeId(file.path, options.root, options.isProd ?? false, file.source),
      });
    });
  }

  const result = compileSfcBatchWithResults(resolvedFiles, buildCompileBatchOptions(options));

  // Released native bindings predating the batch `scopeId` field ignore it and
  // return their absolute-path scope instead. Keep the JS adapter compatible
  // with those bindings by falling back to the older single-file API, which
  // already accepts an explicit scope ID. Updated bindings retain the native
  // parallel path without this extra work.
  const filesByPath = new Map(files.map((file) => [file.path, file]));
  const requestedScopes = new Map(resolvedFiles.map((file) => [file.path, file.scopeId]));
  for (const fileResult of result.results) {
    const requestedScope = requestedScopes.get(fileResult.path);
    const input = filesByPath.get(fileResult.path);
    if (
      fileResult.errors.length > 0 ||
      requestedScope === undefined ||
      fileResult.scopeId === requestedScope ||
      input === undefined
    ) {
      continue;
    }

    isolate(fileResult.path, () => {
      const fallback = compileFile(fileResult.path, cache, options, input.source, {
        logWarnings: false,
      });
      fileResult.code = fallback.code;
      delete fileResult.map;
      if (fallback.map !== undefined) fileResult.map = fallback.map;
      delete fileResult.css;
      if (fallback.css !== undefined) fileResult.css = fallback.css;
      fileResult.scopeId = fallback.scopeId;
      fileResult.hasScoped = fallback.hasScoped;
      delete fileResult.typeProject;
      if (fallback.typeProject !== undefined) fileResult.typeProject = fallback.typeProject;
      delete fileResult.macroArtifacts;
      if (fallback.macroArtifacts !== undefined) {
        fileResult.macroArtifacts = fallback.macroArtifacts;
      }
      delete fileResult.moduleShape;
      if (fallback.moduleShape !== undefined) fileResult.moduleShape = fallback.moduleShape;
    });
  }

  for (const fileResult of result.results) {
    if (fileResult.errors.length > 0 || failed.has(fileResult.path)) continue;
    const imports = inlinedByPath.get(fileResult.path)?.imports;
    isolate(fileResult.path, () =>
      assertSfcStyleSrcImports(fileResult.path, fileResult.styles ?? [], imports),
    );
  }

  // Update cache with results
  for (const fileResult of result.results) {
    if (failed.has(fileResult.path)) continue;
    if (fileResult.errors.length === 0) {
      cache.set(fileResult.path, {
        ...(fileResult.typeProject ? { typeProject: fileResult.typeProject } : {}),
        source: filesByPath.get(fileResult.path)?.source,
        code: fileResult.code,
        ...(fileResult.map ? { map: fileResult.map } : {}),
        css: fileResult.css,
        scopeId: fileResult.scopeId,
        hasScoped: fileResult.hasScoped,
        templateHash: fileResult.templateHash,
        styleHash: fileResult.styleHash,
        scriptHash: fileResult.scriptHash,
        macroArtifacts: fileResult.macroArtifacts ?? [],
        styles: normalizeStyleBlocks(fileResult.styles, inlinedByPath.get(fileResult.path)),
        ...customBlockFields(fileResult.customBlocks),
        dependencies: [
          ...(inlinedByPath.get(fileResult.path)?.dependencies ?? []),
          ...(fileResult.dependencies ?? []),
        ],
        ...(fileResult.moduleShape ? { moduleShape: fileResult.moduleShape } : {}),
      });
    }

    if (!log) continue;
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
