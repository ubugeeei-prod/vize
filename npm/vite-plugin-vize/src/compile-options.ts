import type { BatchCompileOptionsNapi, SfcCompileOptionsNapi } from "./types.ts";
import { generateScopeId } from "./utils/index.ts";

export interface CompileFileOptions {
  sourceMap: boolean;
  ssr: boolean;
  vapor: boolean;
  mode?: "module" | "function";
  customRenderer?: boolean;
  vueParserQuirks?: boolean;
  runtimeModuleName?: string;
  runtimeGlobalName?: string;
  vueVersion?: string | number;
}

export interface CompileBatchOptions {
  ssr: boolean;
  vapor: boolean;
  mode?: "module" | "function";
  customRenderer?: boolean;
  vueParserQuirks?: boolean;
  runtimeModuleName?: string;
  runtimeGlobalName?: string;
  vueVersion?: string | number;
}

export function buildCompileFileOptions(
  filePath: string,
  options: CompileFileOptions,
): SfcCompileOptionsNapi {
  return {
    filename: filePath,
    mode: options.mode,
    sourceMap: options.sourceMap,
    ssr: options.ssr,
    vapor: options.vapor,
    customRenderer: options.customRenderer ?? false,
    vueParserQuirks: options.vueParserQuirks ?? false,
    runtimeModuleName: options.runtimeModuleName,
    runtimeGlobalName: options.runtimeGlobalName,
    vueVersion: options.vueVersion == null ? undefined : String(options.vueVersion),
    scopeId: `data-v-${generateScopeId(filePath)}`,
  };
}

export function buildCompileBatchOptions(options: CompileBatchOptions): BatchCompileOptionsNapi {
  return {
    mode: options.mode,
    ssr: options.ssr,
    vapor: options.vapor,
    customRenderer: options.customRenderer ?? false,
    vueParserQuirks: options.vueParserQuirks ?? false,
    runtimeModuleName: options.runtimeModuleName,
    runtimeGlobalName: options.runtimeGlobalName,
    vueVersion: options.vueVersion == null ? undefined : String(options.vueVersion),
  };
}
