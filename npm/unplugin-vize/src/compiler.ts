import { createHash } from "node:crypto";
import * as native from "@vizejs/native";
import type {
  CompiledModule,
  NormalizedVizeUnpluginOptions,
  CachedCompiledModule,
  SfcCompileResultNapi,
} from "./types.ts";
import { extractStyleBlocks, generateScopeId } from "./style.ts";

const { compileSfc } = native as {
  compileSfc: (source: string, options?: Record<string, unknown>) => SfcCompileResultNapi;
};

function buildSignature(options: NormalizedVizeUnpluginOptions): string {
  return [
    options.isProduction ? "1" : "0",
    options.ssr ? "1" : "0",
    options.vapor ? "1" : "0",
    options.customRenderer ? "1" : "0",
    options.sourceMap ? "1" : "0",
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
  const hasScoped = /<style[^>]*\bscoped\b/.test(source);
  const result = compileSfc(source, {
    filename: filePath,
    sourceMap: options.sourceMap,
    ssr: options.ssr,
    vapor: options.vapor,
    customRenderer: options.customRenderer,
    scopeId: hasScoped ? `data-v-${scopeId}` : undefined,
  });

  if (result.errors.length > 0) {
    throw new Error(result.errors.join("\n"));
  }

  const compiled: CompiledModule = {
    code: result.code,
    css: result.css,
    scopeId,
    hasScoped,
    templateHash: result.templateHash,
    styleHash: result.styleHash,
    scriptHash: result.scriptHash,
    macroArtifacts: result.macroArtifacts ?? [],
    styles: extractStyleBlocks(source),
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
