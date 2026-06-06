import type { BatchCompileOptionsNapi, SfcCompileOptionsNapi } from "./types.ts";
import { generateScopeId } from "./utils/index.ts";

export interface CompileFileOptions {
  sourceMap: boolean;
  ssr: boolean;
  vapor: boolean;
  mode?: "module" | "function";
  customRenderer?: boolean;
  templateSyntax?: "standard" | "strict" | "quirks";
  runtimeModuleName?: string;
  runtimeGlobalName?: string;
  vueVersion?: string | number;
}

export interface CompileBatchOptions {
  ssr: boolean;
  vapor: boolean;
  mode?: "module" | "function";
  customRenderer?: boolean;
  templateSyntax?: "standard" | "strict" | "quirks";
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
    sourceMap: options.sourceMap,
    ssr: options.ssr,
    vapor: options.vapor,
    customRenderer: options.customRenderer ?? false,
    scopeId: `data-v-${generateScopeId(filePath)}`,
    ...(options.mode === undefined ? {} : { mode: options.mode }),
    ...(options.templateSyntax === undefined ? {} : { templateSyntax: options.templateSyntax }),
    ...(options.runtimeModuleName === undefined
      ? {}
      : { runtimeModuleName: options.runtimeModuleName }),
    ...(options.runtimeGlobalName === undefined
      ? {}
      : { runtimeGlobalName: options.runtimeGlobalName }),
    ...(options.vueVersion == null ? {} : { vueVersion: String(options.vueVersion) }),
  };
}

export function buildCompileBatchOptions(options: CompileBatchOptions): BatchCompileOptionsNapi {
  return {
    ssr: options.ssr,
    vapor: options.vapor,
    customRenderer: options.customRenderer ?? false,
    ...(options.mode === undefined ? {} : { mode: options.mode }),
    ...(options.templateSyntax === undefined ? {} : { templateSyntax: options.templateSyntax }),
    ...(options.runtimeModuleName === undefined
      ? {}
      : { runtimeModuleName: options.runtimeModuleName }),
    ...(options.runtimeGlobalName === undefined
      ? {}
      : { runtimeGlobalName: options.runtimeGlobalName }),
    ...(options.vueVersion == null ? {} : { vueVersion: String(options.vueVersion) }),
  };
}
