import { createHash } from "node:crypto";
import * as native from "@vizejs/native";
import type {
  CompiledModule,
  NormalizedVizeUnpluginOptions,
  CachedCompiledModule,
  JsxCompileResultNapi,
  SfcCompileResultNapi,
} from "./types.ts";
import { generateScopeId, toStyleBlockInfo } from "./style.ts";

const { compileSfc, compileJsx } = native as {
  compileSfc: (source: string, options?: Record<string, unknown>) => SfcCompileResultNapi;
  compileJsx: (source: string, options?: Record<string, unknown>) => JsxCompileResultNapi;
};

function buildSignature(options: NormalizedVizeUnpluginOptions): string {
  return [
    options.isProduction ? "1" : "0",
    options.ssr ? "1" : "0",
    options.vapor ? "1" : "0",
    options.customRenderer ? "1" : "0",
    options.templateSyntax,
    options.sourceMap ? "1" : "0",
    options.mode,
    options.runtimeModuleName,
    options.runtimeGlobalName,
    String(options.vueVersion),
    options.hostCompiler ? "1" : "0",
    options.root,
  ].join(":");
}

function buildSourceHash(source: string): string {
  return createHash("sha256").update(source).digest("hex");
}

export function compileVueModule(
  filePath: string,
  source: string,
  options: NormalizedVizeUnpluginOptions,
  cache: Map<string, CachedCompiledModule>,
): { compiled: CompiledModule; warnings: string[] } {
  const sourceHash = buildSourceHash(source);
  const signature = buildSignature(options);
  const cached = cache.get(filePath);

  if (cached && cached.sourceHash === sourceHash && cached.signature === signature) {
    return { compiled: cached.compiled, warnings: [] };
  }

  const scopeId = generateScopeId(filePath, options.root, options.isProduction, source);
  const result = compileSfc(source, {
    filename: filePath,
    mode: options.mode,
    sourceMap: options.sourceMap,
    ssr: options.ssr,
    vapor: options.vapor,
    customRenderer: options.customRenderer,
    templateSyntax: options.templateSyntax,
    runtimeModuleName: options.runtimeModuleName,
    runtimeGlobalName: options.runtimeGlobalName,
    vueVersion: String(options.vueVersion),
    scopeId: `data-v-${scopeId}`,
  });

  if (result.errors.length > 0) {
    throw new Error(result.errors.join("\n"));
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
    styles: result.styles.map(toStyleBlockInfo),
  };

  cache.set(filePath, {
    compiled,
    sourceHash,
    signature,
  });

  return {
    compiled,
    warnings: result.warnings,
  };
}

export function compileJsxModule(
  filePath: string,
  source: string,
  options: NormalizedVizeUnpluginOptions,
): { code: string; warnings: string[] } {
  const result = compileJsx(source, {
    filename: filePath,
    lang: filePath.endsWith(".tsx") ? "tsx" : "jsx",
    vapor: options.vapor,
  });

  if (result.errors.length > 0) {
    throw new Error(result.errors.join("\n"));
  }

  return {
    code: result.code,
    warnings: result.warnings,
  };
}
