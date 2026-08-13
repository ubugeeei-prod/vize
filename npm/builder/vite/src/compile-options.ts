import type { BatchCompileOptionsNapi, SfcCompileOptionsNapi } from "./types.ts";
import { generateScopeId } from "./utils/index.ts";
import type { PluginVueCompileOptions } from "./plugin/plugin-vue-options.ts";

export interface CompileFileOptions extends PluginVueCompileOptions {
  sourceMap: boolean;
  ssr: boolean;
  vapor: boolean;
  mode?: "module" | "function";
  customRenderer?: boolean;
  templateSyntax?: "standard" | "strict" | "quirks";
  experimentalInTagComments?: boolean;
  experimentalPatternedTemplate?: boolean;
  experimentalServerScript?: boolean;
  runtimeModuleName?: string;
  runtimeGlobalName?: string;
  vueVersion?: string | number;
}

export interface CompileBatchOptions extends PluginVueCompileOptions {
  sourceMap: boolean;
  ssr: boolean;
  vapor: boolean;
  mode?: "module" | "function";
  customRenderer?: boolean;
  templateSyntax?: "standard" | "strict" | "quirks";
  experimentalInTagComments?: boolean;
  experimentalPatternedTemplate?: boolean;
  experimentalServerScript?: boolean;
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
    experimentalInTagComments: options.experimentalInTagComments ?? false,
    experimentalPatternedTemplate: options.experimentalPatternedTemplate ?? false,
    experimentalServerScript: options.experimentalServerScript ?? false,
    scopeId: `data-v-${generateScopeId(filePath)}`,
    styleTrim: options.styleTrim,
    ...(options.templateCacheHandlers === undefined
      ? {}
      : { templateCacheHandlers: options.templateCacheHandlers }),
    ...(options.templateComments === undefined
      ? {}
      : { templateComments: options.templateComments }),
    ...(options.templateHoistStatic === undefined
      ? {}
      : { templateHoistStatic: options.templateHoistStatic }),
    ...(options.templatePrefixIdentifiers === undefined
      ? {}
      : { templatePrefixIdentifiers: options.templatePrefixIdentifiers }),
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
    experimentalInTagComments: options.experimentalInTagComments ?? false,
    experimentalPatternedTemplate: options.experimentalPatternedTemplate ?? false,
    experimentalServerScript: options.experimentalServerScript ?? false,
    // Opt into exactly the optional payloads the bundler pipeline consumes:
    // per-block style metadata, macro artifacts, and HMR content hashes.
    // Custom blocks are not used in the batch path, so they stay omitted.
    includeStyles: true,
    includeMacroArtifacts: true,
    includeHashes: true,
    // Also part of the pre-compile cache key, so a run that wants maps cannot
    // be served entries compiled without them (#3399).
    includeSourceMap: options.sourceMap,
    styleTrim: options.styleTrim,
    ...(options.templateCacheHandlers === undefined
      ? {}
      : { templateCacheHandlers: options.templateCacheHandlers }),
    ...(options.templateComments === undefined
      ? {}
      : { templateComments: options.templateComments }),
    ...(options.templateHoistStatic === undefined
      ? {}
      : { templateHoistStatic: options.templateHoistStatic }),
    ...(options.templatePrefixIdentifiers === undefined
      ? {}
      : { templatePrefixIdentifiers: options.templatePrefixIdentifiers }),
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
