import fs from "node:fs";
import path from "node:path";
import { pathToFileURL } from "node:url";
import { classifyVitePluginRequest } from "@vizejs/native";
import type { TransformResult } from "vite";

import {
  getCompileOptionsForRequest as compileOptions,
  getEnvironmentCache,
  shouldExtractCssForRequest,
  syncCollectedCssForFile,
  type VizePluginState,
} from "./state.ts";
import { getLoadableVueSfcPath } from "./load-sfc.ts";
import { normalizeVueServerRendererImport } from "./ssr-modules.ts";
import { compileFile, compileJsxModule } from "../compiler.ts";
import { resolveCssImports } from "../utils/css.ts";
import type { SourceMapV3 } from "../utils/source-map.ts";
import { fromPluginVisibleVirtualId, LEGACY_VIZE_PREFIX, RESOLVED_CSS_MODULE } from "../virtual.ts";
import { transformVizeVirtualModule } from "./vite-transform.ts";
import {
  getCompiledStyleSource,
  getVirtualStyleContent,
  normalizeStyleVirtualId,
} from "./load-style.ts";
import { getSfcModuleRequest, loadSfcModuleSource } from "./sfc-module.ts";

export { normalizeVueServerRendererImport };
export { getBoundaryPlaceholderCode } from "./sfc-module.ts";

type LoadOptions = { ssr?: boolean };

function findMacroArtifactModule(
  state: VizePluginState,
  realPath: string,
  ssr: boolean,
  kind: string,
): string | null {
  const cache = getEnvironmentCache(state, ssr);
  const extractCss = shouldExtractCssForRequest(state, ssr);
  realPath = classifyVitePluginRequest(realPath).normalizedVuePath;
  let compiled = cache.get(realPath) ?? state.cache.get(realPath) ?? state.ssrCache.get(realPath);

  if (!compiled && fs.existsSync(realPath)) {
    const source = fs.readFileSync(realPath, "utf-8");
    compiled = compileFile(realPath, cache, compileOptions(state, ssr, realPath), source);
  }
  syncCollectedCssForFile({ ...state, extractCss }, realPath, compiled);

  return compiled?.macroArtifacts?.find((artifact) => artifact.kind === kind)?.moduleCode ?? null;
}

function loadDefinePageArtifact(
  state: VizePluginState,
  realPath: string,
  ssr: boolean,
): { code: string; map: null } {
  return {
    code:
      findMacroArtifactModule(state, realPath, ssr, "vue-router.definePage") ?? "export default {}",
    map: null,
  };
}

function loadDefinePageMetaArtifact(
  state: VizePluginState,
  realPath: string,
  ssr: boolean,
): { code: string; map: null } | null {
  const code = findMacroArtifactModule(state, realPath, ssr, "nuxt.definePageMeta");
  return code ? { code, map: null } : null;
}

export function loadHook(
  state: VizePluginState,
  id: string,
  loadOptions?: LoadOptions,
): string | { code: string; map: SourceMapV3 | null } | null {
  // A `<style src>` module is the src file; its query names the owning SFC.
  if (
    id !== RESOLVED_CSS_MODULE &&
    !id.startsWith("\0") &&
    !id.includes(".vue") &&
    !id.includes("vize-file=")
  ) {
    return null;
  }

  const request = classifyVitePluginRequest(id);
  const pluginVisibleVirtualPath = fromPluginVisibleVirtualId(id);
  const loadableVueSfcPath = getLoadableVueSfcPath(request);

  // Pick the correct viteBase for URL resolution based on the build environment.
  const currentBase = loadOptions?.ssr ? state.serverViteBase : state.clientViteBase;

  // Handle virtual CSS module for production extraction
  if (id === RESOLVED_CSS_MODULE) {
    let allCss = "";
    for (const css of state.collectedCss.values()) {
      allCss += allCss ? `\n\n${css}` : css;
    }
    return allCss;
  }

  // Strip the \0 prefix and the appended extension suffix for style virtual IDs.
  const styleId = normalizeStyleVirtualId(id);

  const styleRequest = classifyVitePluginRequest(styleId);
  if (styleRequest.isVueStyleQuery) {
    const isRootRelative =
      !!styleRequest.querySuffix &&
      new URLSearchParams(styleRequest.querySuffix.slice(1)).has("vize-root-relative");
    const sourceRequest = classifyVitePluginRequest(styleRequest.path);
    let realPath =
      sourceRequest.vizeVirtualPath ??
      sourceRequest.normalizedFsId ??
      sourceRequest.normalizedVuePath ??
      styleRequest.path;
    if (realPath.startsWith("/") && (isRootRelative || !fs.existsSync(realPath))) {
      const projectPath = path.resolve(state.root, realPath.slice(1));
      const projectCache = getEnvironmentCache(state, !!loadOptions?.ssr);
      if (
        fs.existsSync(projectPath) ||
        projectCache.has(projectPath) ||
        state.cache.has(projectPath) ||
        state.ssrCache.has(projectPath)
      ) {
        realPath = projectPath;
      }
    }
    const lang = styleRequest.styleLang ?? null;
    const scoped = styleRequest.styleScoped ?? null;
    const fallbackCompiled = getCompiledStyleSource(state, realPath, !!loadOptions?.ssr);
    const blockIndex = styleRequest.styleIndex ?? -1;

    if (
      fallbackCompiled?.styles &&
      blockIndex >= 0 &&
      blockIndex < fallbackCompiled.styles.length
    ) {
      const block = fallbackCompiled.styles[blockIndex];
      return {
        code: getVirtualStyleContent(block, {
          scopeId: fallbackCompiled.scopeId,
          scoped,
          lang,
          trim: state.mergedOptions.style?.trim,
          productionFilename: state.isProduction ? realPath : undefined,
        }),
        map: null,
      };
    }

    if (fallbackCompiled?.css) {
      return resolveCssImports(
        fallbackCompiled.css,
        realPath,
        state.cssAliasRules,
        state.server !== null,
        currentBase,
      );
    }
    return null;
  }

  if (
    id !== RESOLVED_CSS_MODULE &&
    !id.startsWith("\0") &&
    !pluginVisibleVirtualPath &&
    !loadableVueSfcPath
  ) {
    return null;
  }

  // Handle Vue Router's ?definePage query through extracted artifacts.
  if (id.startsWith("\0") && request.hasDefinePageQuery) {
    const realPath = request.strippedVirtualPath ?? "";
    if (request.isVueSfcPath) {
      return loadDefinePageArtifact(state, realPath, !!loadOptions?.ssr);
    }
  }

  // Handle ?macro=true queries
  if (id.startsWith("\0") && request.hasMacroQuery) {
    const realPath = request.strippedVirtualPath ?? "";
    if (request.isVueSfcPath) {
      const artifactLoad = loadDefinePageMetaArtifact(state, realPath, !!loadOptions?.ssr);
      if (artifactLoad) {
        return artifactLoad;
      }
      return { code: "export default {}", map: null };
    }
  }

  // An SFC main module enters the pipeline as its SFC text; `transform` compiles it.
  if (request.isVizeVirtual || pluginVisibleVirtualPath || loadableVueSfcPath) {
    const sfc = getSfcModuleRequest(state, id);
    return sfc ? loadSfcModuleSource(state, sfc, !!loadOptions?.ssr) : null;
  }

  // Handle \0-prefixed non-vue files leaked from virtual module dynamic imports.
  if (id.startsWith("\0")) {
    const afterPrefix = id.startsWith(LEGACY_VIZE_PREFIX)
      ? id.slice(LEGACY_VIZE_PREFIX.length)
      : id.slice(1);
    if (afterPrefix.includes("?commonjs-")) {
      return null;
    }
    const leakedRequest = classifyVitePluginRequest(afterPrefix);
    const fsPath = leakedRequest.normalizedFsId
      ? classifyVitePluginRequest(leakedRequest.normalizedFsId).path
      : leakedRequest.path;
    const querySuffix = leakedRequest.querySuffix;
    if (fsPath.startsWith("/") && fs.existsSync(fsPath) && fs.statSync(fsPath).isFile()) {
      const importPath =
        state.server === null
          ? `${pathToFileURL(fsPath).href}${querySuffix}`
          : "/@fs" + fsPath + querySuffix;
      state.logger.log(`load: proxying \0-prefixed file ${id} -> re-export from ${importPath}`);
      return `export { default } from ${JSON.stringify(importPath)};\nexport * from ${JSON.stringify(importPath)};`;
    }
  }

  return null;
}

function isJsxComponentPath(path: string): boolean {
  return /\.[jt]sx$/.test(path) && !/\.(?:stories|story)\.[jt]sx$/.test(path);
}

function shouldTransformJsxRequest(
  state: VizePluginState,
  request: ReturnType<typeof classifyVitePluginRequest>,
): boolean {
  if (!isJsxComponentPath(request.path)) {
    return false;
  }
  // Skip Vite asset/worker imports of a JSX file (?raw, ?url, ?worker), which
  // must keep Vite's default handling rather than being compiled to render code.
  if (request.querySuffix) {
    const params = new URLSearchParams(request.querySuffix.slice(1));
    if (
      params.has("raw") ||
      params.has("url") ||
      params.has("worker") ||
      params.has("sharedworker")
    ) {
      return false;
    }
  }
  // JSX/TSX modules are valid plain Vite inputs; only compile explicit includes.
  if (!state.mergedOptions.include || !state.filter(request.path)) {
    return false;
  }
  return true;
}

/**
 * Route a raw `.jsx`/`.tsx` request through Vize's JSX compiler.
 *
 * Returns `undefined` when the request is not a JSX/TSX component (so the
 * caller falls through to the virtual-module pipeline), or a transform result
 * otherwise.
 */
export function transformJsxRequest(
  state: VizePluginState,
  code: string,
  id: string,
  options?: { ssr?: boolean },
): TransformResult | undefined {
  const request = classifyVitePluginRequest(id);
  if (!shouldTransformJsxRequest(state, request)) {
    return undefined;
  }

  const realPath = request.normalizedFsId
    ? classifyVitePluginRequest(request.normalizedFsId).path
    : request.path;

  const ssr = options?.ssr ?? false;
  // Match the SFC path's source-map policy: on unless explicitly disabled or in
  // a production build (#1533).
  const fileOptions = state.fileCompilerOptions?.(realPath) ?? state.mergedOptions;

  const {
    code: compiled,
    map,
    warnings,
  } = compileJsxModule(realPath, code, {
    jsxMode: fileOptions.jsxMode,
    jsxCompat: fileOptions.jsxCompat,
    vapor: fileOptions.vapor ?? false,
    ssr,
    sourceMap: compileOptions(state, ssr, realPath).sourceMap,
  });
  for (const warning of warnings) {
    state.logger.warn(`Warning in ${realPath}: ${warning}`);
  }

  // JSX lacks SFC-style HMR records (#1533); keep module reload and parse its v3 map for Vite.
  return { code: compiled, map: map ? JSON.parse(map) : null };
}

// Strip TypeScript from compiled .vue output and apply define replacements
export async function transformHook(
  state: VizePluginState,
  code: string,
  id: string,
  options?: { ssr?: boolean },
): Promise<TransformResult | null> {
  if (!id.startsWith("\0") && !id.includes(".vue") && !isJsxComponentPath(id)) return null;

  const pluginVisibleVirtualPath = fromPluginVisibleVirtualId(id);

  // Compile `.jsx`/`.tsx` Vue components through Vize. Unlike SFCs, JSX/TSX
  // modules are real files Vite hands directly to the transform hook, so they
  // are handled here rather than through the virtual-module load pipeline.
  const jsxResult = transformJsxRequest(state, code, id, { ssr: options?.ssr });
  if (jsxResult !== undefined) {
    return jsxResult;
  }

  if (!id.startsWith("\0") && !pluginVisibleVirtualPath) {
    return null;
  }

  const request = classifyVitePluginRequest(id);
  // An SFC main module is still SFC text here; its compile stage applies these.
  if (!request.isMacroVirtualId && getSfcModuleRequest(state, id)) return null;
  if (request.isVizeVirtual || request.isMacroVirtualId || pluginVisibleVirtualPath) {
    const realPath = request.isMacroVirtualId
      ? (request.strippedVirtualPath ?? "")
      : (request.vizeVirtualPath ?? pluginVisibleVirtualPath ?? "");
    return transformVizeVirtualModule(
      state,
      code,
      realPath,
      options?.ssr ?? false,
      request.isMacroVirtualId,
    );
  }

  return null;
}
