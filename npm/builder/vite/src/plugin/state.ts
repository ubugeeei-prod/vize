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
  fileCompilerOptions?: (file: string) => VizeOptions;
  compilerScopeIdentity?: unknown;
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
  nuxtPageMeta?: boolean;
  mode?: "module" | "function";
  customRenderer: boolean;
  customElements?: string[];
  templateSyntax: "standard" | "strict" | "quirks";
  runtimeModuleName?: string;
  runtimeGlobalName?: string;
  vueVersion?: string | number;
  inlineTemplate?: boolean;
  isProd?: boolean;
} & PluginVueCompileOptions &
  Partial<
    Pick<
      VizeOptions,
      | "experimentalInTagComments"
      | "experimentalPatternedTemplate"
      | "experimentalSelfComponent"
      | "experimentalStrictSlotChildren"
      | "experimentalServerScript"
    >
  >;

export function getCompileOptionsForRequest(
  state: Pick<
    VizePluginState,
    "isProduction" | "mergedOptions" | "viteBuildSourcemap" | "fileCompilerOptions"
  >,
  ssr: boolean,
  file?: string,
): CompileOptionsForRequest {
  const mergedOptions =
    file === undefined
      ? state.mergedOptions
      : (state.fileCompilerOptions?.(file) ?? state.mergedOptions);
  const options: CompileOptionsForRequest = {
    sourceMap: mergedOptions?.sourceMap ?? (!state.isProduction || !!state.viteBuildSourcemap),
    ssr,
    // Vapor runtime is client-oriented today; use VDOM for SSR and Vapor on the client.
    vapor: !ssr && (mergedOptions?.vapor ?? false),
    customRenderer: mergedOptions?.customRenderer ?? false,
    templateSyntax: mergedOptions?.templateSyntax ?? "standard",
    // Vue keeps template comments in development and drops them in production.
    // An explicit template.compilerOptions.comments overrides this default.
    templateComments: !state.isProduction,
    ...resolvePluginVueCompileOptions(mergedOptions ?? {}),
  };

  if (mergedOptions?.nuxtPageMeta !== undefined) {
    options.nuxtPageMeta = mergedOptions.nuxtPageMeta;
  }
  if (mergedOptions?.customElements !== undefined) {
    options.customElements = mergedOptions.customElements;
  }
  if (mergedOptions?.mode !== undefined) {
    options.mode = mergedOptions.mode;
  }
  if (mergedOptions?.runtimeModuleName !== undefined) {
    options.runtimeModuleName = mergedOptions.runtimeModuleName;
  }
  if (mergedOptions?.runtimeGlobalName !== undefined) {
    options.runtimeGlobalName = mergedOptions.runtimeGlobalName;
  }
  if (mergedOptions?.vueVersion !== undefined) {
    options.vueVersion = mergedOptions.vueVersion;
  }
  if (mergedOptions?.experimentalInTagComments) {
    options.experimentalInTagComments = true;
  }
  if (mergedOptions?.experimentalPatternedTemplate) {
    options.experimentalPatternedTemplate = true;
  }
  if (mergedOptions?.experimentalSelfComponent) {
    options.experimentalSelfComponent = true;
  }
  if (mergedOptions?.experimentalStrictSlotChildren) {
    options.experimentalStrictSlotChildren = true;
  }
  if (mergedOptions?.experimentalServerScript) {
    options.experimentalServerScript = true;
  }

  // Client production matches @vitejs/plugin-vue: inline the render into setup
  // and drop dev-only prop runtime types. SSR keeps a separate render. A
  // standalone `mode: "function"` compile already chooses its own shape.
  if (state.isProduction) {
    options.isProd = true;
    if (!ssr && options.mode !== "function") {
      options.inlineTemplate = true;
    }
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
