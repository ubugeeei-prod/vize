import type { BatchCompileOptionsNapi, SfcCompileOptionsNapi } from "./types.ts";
import { generateScopeId } from "./utils/index.ts";

export interface CompileFileOptions {
  sourceMap: boolean;
  ssr: boolean;
  vapor: boolean;
  customRenderer?: boolean;
}

export interface CompileBatchOptions {
  ssr: boolean;
  vapor: boolean;
  customRenderer?: boolean;
}

export function buildCompileFileOptions(
  filePath: string,
  source: string,
  options: CompileFileOptions,
): SfcCompileOptionsNapi {
  const scopeId = /<style[^>]*\bscoped\b/.test(source)
    ? `data-v-${generateScopeId(filePath)}`
    : undefined;

  return {
    filename: filePath,
    sourceMap: options.sourceMap,
    ssr: options.ssr,
    vapor: options.vapor,
    customRenderer: options.customRenderer ?? false,
    scopeId,
  };
}

export function buildCompileBatchOptions(options: CompileBatchOptions): BatchCompileOptionsNapi {
  return {
    ssr: options.ssr,
    vapor: options.vapor,
    customRenderer: options.customRenderer ?? false,
  };
}
