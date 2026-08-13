/**
 * Plugin state type and the caches derived from it.
 *
 * The pre-compilation pass itself lives in `./precompile-run.ts`.
 */

import type { ViteDevServer } from "vite";

import type { VizeOptions, CompiledModule } from "../types.ts";
import { resolveCssImports, type CssAliasRule } from "../utils/css.ts";
import { hasDelegatedStyles } from "../utils/index.ts";
import { type DynamicImportAliasRule } from "../virtual.ts";
import { createLogger } from "../transform.ts";
import type { HmrUpdateType } from "../hmr.ts";
import type { PrecompileFileMetadata } from "./precompile.ts";
import {
  resolvePluginVueCompileOptions,
  type PluginVueCompileOptions,
} from "./plugin-vue-options.ts";

export {
  DEFAULT_PRECOMPILE_BATCH_MAX_BYTES,
  DEFAULT_PRECOMPILE_BATCH_SIZE,
  DEFAULT_PRECOMPILE_IGNORE_PATTERNS,
  chunkPrecompileFiles,
  diffPrecompileFiles,
  hasFileMetadataChanged,
  isPrecompileSfcPath,
  normalizePrecompileBatchSize,
  type PrecompileChunkOptions,
  type PrecompileDiff,
  type PrecompileFileMetadata,
} from "./precompile.ts";

export interface VizePluginState {
  cache: Map<string, CompiledModule>;
  ssrCache: Map<string, CompiledModule>;
  collectedCss: Map<string, string>;
  precompileMetadata: Map<string, PrecompileFileMetadata>;
  pendingHmrUpdateTypes: Map<string, HmrUpdateType>;
  viteResolveCache?: Map<string, Promise<{ id: string; external?: boolean } | null>>;
  isProduction: boolean;
  /**
   * Whether Vite's own `build.sourcemap` asks for source maps (#3399).
   *
   * The documented compiler default is "development on, production off", but a
   * production build that turns `build.sourcemap` on is asking for exactly the
   * maps that default would suppress, so it overrides the production half.
   *
   * Optional so a caller that builds a state literal without it keeps the
   * previous behaviour (dev on, production off) rather than failing to compile.
   */
  viteBuildSourcemap?: boolean;
  root: string;
  clientViteBase: string;
  serverViteBase: string;
  server: ViteDevServer | null;
  filter: (id: string) => boolean;
  scanPatterns: string[] | null;
  precompileBatchSize: number;
  ignorePatterns: string[];
  mergedOptions: VizeOptions;
  initialized: boolean;
  dynamicImportAliasRules: DynamicImportAliasRule[];
  cssAliasRules: CssAliasRule[];
  extractCss: boolean;
  componentsCssFileName: string;
  clientViteDefine: Record<string, string>;
  serverViteDefine: Record<string, string>;
  logger: ReturnType<typeof createLogger>;
}

export function getEnvironmentCache(
  state: Pick<VizePluginState, "cache" | "ssrCache">,
  ssr: boolean,
): Map<string, CompiledModule> {
  return ssr ? state.ssrCache : state.cache;
}

export type CompileOptionsForRequest = {
  sourceMap: boolean;
  ssr: boolean;
  vapor: boolean;
  mode?: "module" | "function";
  customRenderer: boolean;
  templateSyntax: "standard" | "strict" | "quirks";
  runtimeModuleName?: string;
  runtimeGlobalName?: string;
  vueVersion?: string | number;
} & PluginVueCompileOptions &
  Partial<
    Pick<
      VizeOptions,
      "experimentalInTagComments" | "experimentalPatternedTemplate" | "experimentalServerScript"
    >
  >;

export function getCompileOptionsForRequest(
  state: Pick<VizePluginState, "isProduction" | "mergedOptions" | "viteBuildSourcemap">,
  ssr: boolean,
): CompileOptionsForRequest {
  const options: CompileOptionsForRequest = {
    sourceMap:
      state.mergedOptions?.sourceMap ?? (!state.isProduction || !!state.viteBuildSourcemap),
    ssr,
    // Vapor runtime is client-oriented today; use VDOM for SSR and Vapor on the client.
    vapor: !ssr && (state.mergedOptions?.vapor ?? false),
    customRenderer: state.mergedOptions?.customRenderer ?? false,
    templateSyntax: state.mergedOptions?.templateSyntax ?? "standard",
    ...resolvePluginVueCompileOptions(state.mergedOptions ?? {}),
  };

  if (state.mergedOptions?.mode !== undefined) {
    options.mode = state.mergedOptions.mode;
  }
  if (state.mergedOptions?.runtimeModuleName !== undefined) {
    options.runtimeModuleName = state.mergedOptions.runtimeModuleName;
  }
  if (state.mergedOptions?.runtimeGlobalName !== undefined) {
    options.runtimeGlobalName = state.mergedOptions.runtimeGlobalName;
  }
  if (state.mergedOptions?.vueVersion !== undefined) {
    options.vueVersion = state.mergedOptions.vueVersion;
  }
  if (state.mergedOptions?.experimentalInTagComments) {
    options.experimentalInTagComments = true;
  }
  if (state.mergedOptions?.experimentalPatternedTemplate) {
    options.experimentalPatternedTemplate = true;
  }
  if (state.mergedOptions?.experimentalServerScript) {
    options.experimentalServerScript = true;
  }

  return options;
}

export function syncCollectedCssForFile(
  state: Pick<VizePluginState, "extractCss" | "collectedCss" | "cssAliasRules">,
  filePath: string,
  compiled: CompiledModule | undefined,
): void {
  if (!compiled || !state.extractCss) {
    return;
  }

  if (compiled.styles?.length) {
    state.collectedCss.delete(filePath);
    return;
  }

  if (compiled.css && !hasDelegatedStyles(compiled)) {
    state.collectedCss.set(
      filePath,
      resolveCssImports(compiled.css, filePath, state.cssAliasRules, false),
    );
  } else {
    state.collectedCss.delete(filePath);
  }
}

export function shouldExtractCssForRequest(
  state: Pick<VizePluginState, "isProduction">,
  ssr: boolean,
): boolean {
  return state.isProduction && !ssr;
}

export function clearBuildCaches(
  state: Pick<
    VizePluginState,
    | "cache"
    | "collectedCss"
    | "pendingHmrUpdateTypes"
    | "precompileMetadata"
    | "ssrCache"
    | "viteResolveCache"
  >,
): void {
  state.cache.clear();
  state.ssrCache.clear();
  state.collectedCss.clear();
  state.precompileMetadata.clear();
  state.pendingHmrUpdateTypes.clear();
  state.viteResolveCache?.clear();
}
