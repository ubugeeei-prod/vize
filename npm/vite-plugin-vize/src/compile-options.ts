import type { BatchCompileOptionsNapi, SfcCompileOptionsNapi } from "./types.ts";
import { generateScopeId } from "./utils/index.ts";

export interface CompileFileOptions {
  sourceMap: boolean;
  ssr: boolean;
  vapor: boolean;
  customRenderer?: boolean;
  vueParserQuirks?: boolean;
}

export interface CompileBatchOptions {
  ssr: boolean;
  vapor: boolean;
  customRenderer?: boolean;
  vueParserQuirks?: boolean;
}

export function buildCompileFileOptions(
  filePath: string,
  options: CompileFileOptions,
): SfcCompileOptionsNapi {
  return {
    filename: filePath,
    sourceMap: options.sourceMap,
    ssr: options.ssr,
    vapor: options.vapor,
    customRenderer: options.customRenderer ?? false,
    vueParserQuirks: options.vueParserQuirks ?? false,
    scopeId: `data-v-${generateScopeId(filePath)}`,
  };
}

export function buildCompileBatchOptions(options: CompileBatchOptions): BatchCompileOptionsNapi {
  return {
    ssr: options.ssr,
    vapor: options.vapor,
    customRenderer: options.customRenderer ?? false,
    vueParserQuirks: options.vueParserQuirks ?? false,
  };
}
