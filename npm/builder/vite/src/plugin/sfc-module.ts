/**
 * The SFC main module: `load` serves the SFC text and `transform` compiles the
 * text the pipeline delivers, at @vitejs/plugin-vue's position. Host transforms
 * ordered before the compiler (Nuxt's `<DevOnly>` strip, #11641) therefore
 * reach the source, and those after it see the compiled module.
 */

import fs from "node:fs";
import { classifyVitePluginRequest } from "@vizejs/native";

import type { HostResolve } from "../../../shared/sfc-src.ts";
import {
  getCompileOptionsForRequest as compileOptions,
  getEnvironmentCache,
  shouldExtractCssForRequest,
  syncCollectedCssForFile,
  type VizePluginState,
} from "./state.ts";
import { getLoadableVueSfcPath, shouldLoadCompiledVueSfcPath } from "./load-sfc.ts";
import { normalizeStyleVirtualId } from "./style-request.ts";
import { registerSfcModule, normalizeVueServerRendererImport } from "./ssr-modules.ts";
import { prepareSfcSrcRequests } from "./src-requests.ts";
import { compileFile } from "../compiler.ts";
import { embedsInlineCss, generateOutputWithMap, hasDelegatedStyles } from "../utils/index.ts";
import { identitySourceMap, MappedModule, type SourceMapV3 } from "../utils/source-map.ts";
import { resolveCssImports } from "../utils/css.ts";
import { fromPluginVisibleVirtualId, isPluginVisibleSsrVirtualId } from "../virtual.ts";
import {
  rewriteDynamicTemplateImports,
  rewriteImportMetaGlobBase,
  rewriteStaticAssetUrls,
} from "../transform.ts";
import { transformVizeVirtualModule } from "./vite-transform.ts";
import { isPluginVueCustomElement } from "./plugin-vue-options.ts";
import { loadedSfcModule } from "./tree-shaking.ts";

const SERVER_PLACEHOLDER_CODE = `import { createElementBlock, defineComponent } from "vue";
export default defineComponent({
  name: "ServerPlaceholder",
  render() {
    return createElementBlock("div");
  }
});
`;

export function getBoundaryPlaceholderCode(realPath: string, ssr: boolean): string | null {
  const boundaryKind = classifyVitePluginRequest(realPath).boundaryKind;
  if (ssr && boundaryKind === "client") {
    return SERVER_PLACEHOLDER_CODE;
  }
  if (!ssr && boundaryKind === "server") {
    return SERVER_PLACEHOLDER_CODE;
  }
  return null;
}

export interface SfcModuleRequest {
  realPath: string;
  /** The id itself names the SSR graph (`vize-ssr`, the legacy SSR prefix). */
  ssrId: boolean;
  /** A Vize virtual id, which Vite's define pass may skip. */
  virtual: boolean;
  /** Legacy `\0` ids keep the #884 glob base rewrite. */
  nullPrefixed: boolean;
}

/** The SFC an id compiles as a whole component, or `null` for any other request. */
export function getSfcModuleRequest(state: VizePluginState, id: string): SfcModuleRequest | null {
  if (!id.includes(".vue")) return null;
  if (classifyVitePluginRequest(normalizeStyleVirtualId(id)).isVueStyleQuery) return null;
  const request = classifyVitePluginRequest(id);
  const nullPrefixed = id.startsWith("\0");
  if (
    nullPrefixed &&
    request.isVueSfcPath &&
    (request.hasDefinePageQuery || request.hasMacroQuery)
  ) {
    return null;
  }

  const visiblePath = fromPluginVisibleVirtualId(id);
  if (request.isVizeVirtual || visiblePath) {
    const realPath = request.vizeVirtualPath ?? visiblePath ?? "";
    if (!realPath.endsWith(".vue") || !shouldLoadCompiledVueSfcPath(state, realPath)) return null;
    return {
      realPath,
      ssrId: request.isVizeSsrVirtual || isPluginVisibleSsrVirtualId(id),
      virtual: true,
      nullPrefixed,
    };
  }

  const realPath = getLoadableVueSfcPath(request);
  if (!realPath) return null;
  const hasNuxtComponentQuery =
    !!request.querySuffix &&
    new URLSearchParams(request.querySuffix.slice(1)).has("nuxt_component");
  if (!shouldLoadCompiledVueSfcPath(state, realPath, hasNuxtComponentQuery)) return null;
  return { realPath, ssrId: false, virtual: false, nullPrefixed };
}

/**
 * The SFC text a main module request starts the pipeline with, mapped to its
 * file when the compile emits a map.
 */
export function loadSfcModuleSource(
  state: VizePluginState,
  sfc: SfcModuleRequest,
  ssr: boolean,
): string | { code: string; map: SourceMapV3 } | null {
  let source: string;
  try {
    source = fs.readFileSync(sfc.realPath, "utf-8");
  } catch (error) {
    if ((error as NodeJS.ErrnoException).code === "ENOENT") return null;
    throw error;
  }
  if (!compileOptions(state, sfc.ssrId || ssr, sfc.realPath).sourceMap) return source;
  return { code: source, map: identitySourceMap(sfc.realPath, source) };
}

function shouldLogSfcWarnings(state: VizePluginState, realPath: string): boolean {
  if (!realPath.includes("node_modules")) return true;
  if ((state.mergedOptions.handleNodeModulesVue ?? true) === false) return false;
  return state.filter(realPath);
}

export type SfcModuleResult = { code: string; map: SourceMapV3 | null; moduleSideEffects?: false };

export interface SfcModuleTransformOptions {
  ssr?: boolean;
  addWatchFile?: (id: string) => void;
  resolve?: HostResolve;
}

/** Compile the SFC text `code` that the pipeline delivered for `sfc`. */
export async function transformSfcModule(
  state: VizePluginState,
  code: string,
  sfc: SfcModuleRequest,
  options: SfcModuleTransformOptions = {},
): Promise<SfcModuleResult> {
  const { realPath } = sfc;
  const environmentSsr = !!options.ssr;
  const placeholderCode = getBoundaryPlaceholderCode(realPath, environmentSsr);
  if (placeholderCode) {
    state.logger.log(`transform: using boundary placeholder for ${realPath}`);
    return { code: placeholderCode, map: null };
  }

  const isSsr = sfc.ssrId || environmentSsr;
  const currentBase = environmentSsr ? state.serverViteBase : state.clientViteBase;
  const cache = getEnvironmentCache(state, isSsr);
  const customElement = isPluginVueCustomElement(state.mergedOptions, realPath);
  const extractCss = shouldExtractCssForRequest(state, isSsr) && !customElement;
  let compiled = cache.get(realPath);
  if (compiled?.source !== code) {
    state.logger.log(`transform: compiling ${realPath}`);
    if (options.resolve) await prepareSfcSrcRequests(state, options.resolve, realPath, code);
    compiled = compileFile(realPath, cache, compileOptions(state, isSsr, realPath), code, {
      logWarnings: shouldLogSfcWarnings(state, realPath),
    });
  }
  syncCollectedCssForFile({ ...state, extractCss }, realPath, compiled);

  // A transform's watch file is a module import; only a legacy `\0` id needs its own file.
  const dependencies = compiled.dependencies ?? [];
  for (const watchFile of new Set(sfc.nullPrefixed ? [realPath, ...dependencies] : dependencies)) {
    options.addWatchFile?.(watchFile);
  }

  const hasDelegated = hasDelegatedStyles(compiled);
  const outputOptions = {
    isProduction: state.isProduction,
    isDev: state.server !== null && !isSsr,
    ssr: isSsr,
    hmrUpdateType: environmentSsr ? undefined : state.pendingHmrUpdateTypes.get(realPath),
    extractCss,
    customElement,
    filePath: realPath,
    root: state.root,
  };
  if (compiled.css && !hasDelegated && embedsInlineCss(compiled, outputOptions)) {
    compiled = {
      ...compiled,
      css: resolveCssImports(
        compiled.css,
        realPath,
        state.cssAliasRules,
        state.server !== null,
        currentBase,
      ),
    };
  }
  const emitted = generateOutputWithMap(compiled, outputOptions);
  // Each rewrite below substitutes inside a single line, but they go through
  // `MappedModule` anyway so the map cannot silently drift if one ever does not.
  const rewritten = new MappedModule(
    isSsr ? normalizeVueServerRendererImport(emitted.code) : emitted.code,
    emitted.map,
  );
  rewritten.edit(rewriteDynamicTemplateImports(rewritten.code, state.dynamicImportAliasRules));
  rewritten.edit(rewriteStaticAssetUrls(rewritten.code, state.dynamicImportAliasRules));
  // File-based IDs let Vite retain authored glob keys and resolve outside root.
  // Only legacy null-prefixed modules still need the #884 compatibility rewrite.
  if (sfc.nullPrefixed) {
    rewritten.edit(rewriteImportMetaGlobBase(rewritten.code, realPath, state.root));
  }
  rewritten.edit(registerSfcModule(state, rewritten.code, realPath, isSsr));
  const result = loadedSfcModule(
    rewritten.code,
    rewritten.map,
    state.isProduction,
    isSsr,
    compiled,
  );
  if (!sfc.virtual) return result;
  // Define replacements stay within a line, so the compiled map still holds.
  const defined = await transformVizeVirtualModule(state, result.code, realPath, environmentSsr);
  return defined ? { ...result, code: defined.code } : result;
}
