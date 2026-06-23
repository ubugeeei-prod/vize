import fs from "node:fs";
import path from "node:path";
import { pathToFileURL } from "node:url";
import { classifyVitePluginRequest } from "@vizejs/native";
import type { TransformResult } from "vite";

import {
  getCompileOptionsForRequest,
  getEnvironmentCache,
  shouldExtractCssForRequest,
  syncCollectedCssForFile,
  type VizePluginState,
} from "./state.ts";
import { getLoadableVueSfcPath, shouldLoadCompiledVueSfcPath } from "./load-sfc.ts";
import { compileFile, compileJsxModule } from "../compiler.ts";
import { generateOutput, hasDelegatedStyles } from "../utils/index.ts";
import {
  resolveCssImports,
  scopeCssForPipeline,
  transformCssVarsForPipeline,
} from "../utils/css.ts";
import {
  fromPluginVisibleVirtualId,
  isPluginVisibleSsrVirtualId,
  LEGACY_VIZE_PREFIX,
  RESOLVED_CSS_MODULE,
  rewriteDynamicTemplateImports,
} from "../virtual.ts";
import {
  applyDefineReplacements,
  rewriteImportMetaGlobBase,
  rewriteStaticAssetUrls,
} from "../transform.ts";
import { transformVirtualTypeScript } from "./vite-transform.ts";

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

function getOxcDumpPath(root: string, realPath: string): string {
  const dumpDir = path.resolve(root || process.cwd(), "node_modules", ".vize", "oxc-dumps");
  fs.mkdirSync(dumpDir, { recursive: true });
  return path.join(dumpDir, `vize-oxc-error-${path.basename(realPath)}.ts`);
}

function getVirtualModuleDefines(
  state: Pick<VizePluginState, "clientViteDefine" | "isProduction" | "serverViteDefine">,
  ssr: boolean,
): Record<string, string> {
  return {
    "import.meta.client": ssr ? "false" : "true",
    "import.meta.server": ssr ? "true" : "false",
    "import.meta.dev": state.isProduction ? "false" : "true",
    "import.meta.test": "false",
    "import.meta.prerender": "false",
    ...(ssr ? state.serverViteDefine : state.clientViteDefine),
  };
}

export function normalizeVueServerRendererImport(code: string): string {
  return code.replace(/\bfrom\s+(['"])@vue\/server-renderer\1/g, 'from "vue/server-renderer"');
}

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
    compiled = compileFile(realPath, cache, getCompileOptionsForRequest(state, ssr), source);
  }
  syncCollectedCssForFile({ ...state, extractCss }, realPath, compiled);

  return compiled?.macroArtifacts?.find((artifact) => artifact.kind === kind)?.moduleCode ?? null;
}

function normalizeStyleVirtualId(id: string): string {
  const withoutPrefix = id.startsWith("\0") ? id.slice(1) : id;
  if (!withoutPrefix.includes("?vue")) {
    return id;
  }

  return withoutPrefix.replace(/\.module\.\w+$/, "").replace(/\.\w+$/, "");
}

function loadCompiledSfcModule(
  state: VizePluginState,
  realPath: string,
  isSsr: boolean,
  currentBase: string,
  loadOptions?: { ssr?: boolean; addWatchFile?: (id: string) => void },
): { code: string; map: null } | string | null {
  const placeholderCode = getBoundaryPlaceholderCode(realPath, !!loadOptions?.ssr);
  if (placeholderCode) {
    state.logger.log(`load: using boundary placeholder for ${realPath}`);
    return {
      code: placeholderCode,
      map: null,
    };
  }

  const cache = getEnvironmentCache(state, isSsr);
  const extractCss = shouldExtractCssForRequest(state, isSsr);
  let compiled = cache.get(realPath);

  // On-demand compile if not cached
  if (!compiled && fs.existsSync(realPath)) {
    state.logger.log(`load: on-demand compiling ${realPath}`);
    compiled = compileFile(realPath, cache, getCompileOptionsForRequest(state, isSsr), undefined, {
      logWarnings: shouldLogSfcWarnings(state, realPath),
    });
  }
  syncCollectedCssForFile({ ...state, extractCss }, realPath, compiled);

  if (!compiled) {
    return null;
  }

  for (const watchFile of new Set([realPath, ...(compiled.dependencies ?? [])])) {
    loadOptions?.addWatchFile?.(watchFile);
  }

  const hasDelegated = hasDelegatedStyles(compiled);
  const pendingHmrUpdateType = loadOptions?.ssr
    ? undefined
    : state.pendingHmrUpdateTypes.get(realPath);
  if (compiled.css && !hasDelegated) {
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
  const generatedOutput = generateOutput(compiled, {
    isProduction: state.isProduction,
    isDev: state.server !== null && !isSsr,
    ssr: isSsr,
    hmrUpdateType: pendingHmrUpdateType,
    extractCss,
    filePath: realPath,
  });
  const output = rewriteStaticAssetUrls(
    rewriteDynamicTemplateImports(
      isSsr ? normalizeVueServerRendererImport(generatedOutput) : generatedOutput,
      state.dynamicImportAliasRules,
    ),
    state.dynamicImportAliasRules,
  );
  const normalizedOutput = rewriteImportMetaGlobBase(output, realPath, state.root);
  if (!loadOptions?.ssr) {
    state.pendingHmrUpdateTypes.delete(realPath);
  }
  return {
    code: normalizedOutput,
    map: null,
  };
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
  loadOptions?: { ssr?: boolean; addWatchFile?: (id: string) => void },
): string | { code: string; map: null } | null {
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
    const sourceRequest = classifyVitePluginRequest(styleRequest.path);
    const realPath =
      sourceRequest.vizeVirtualPath ??
      sourceRequest.normalizedFsId ??
      sourceRequest.normalizedVuePath ??
      styleRequest.path;
    const lang = styleRequest.styleLang ?? null;
    const scoped = styleRequest.styleScoped ?? null;
    const fallbackCompiled = loadOptions?.ssr
      ? (state.ssrCache.get(realPath) ?? state.cache.get(realPath))
      : (state.cache.get(realPath) ?? state.ssrCache.get(realPath));
    const blockIndex = styleRequest.styleIndex ?? -1;

    if (
      fallbackCompiled?.styles &&
      blockIndex >= 0 &&
      blockIndex < fallbackCompiled.styles.length
    ) {
      const block = fallbackCompiled.styles[blockIndex];
      let styleContent = block.content;

      // Keep delegated plain CSS scoped while preserving PostCSS-only syntax
      // such as `@apply` for the downstream CSS pipeline.
      if (scoped && block.scoped && (!lang || lang === "css")) {
        styleContent = scopeCssForPipeline(styleContent, scoped);
      }

      styleContent = transformCssVarsForPipeline(styleContent, fallbackCompiled.scopeId);

      return {
        code: styleContent,
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

  // Handle vize virtual modules
  if (request.isVizeVirtual || pluginVisibleVirtualPath) {
    const realPath = request.vizeVirtualPath ?? pluginVisibleVirtualPath ?? "";
    const isSsr = request.isVizeSsrVirtual || isPluginVisibleSsrVirtualId(id) || !!loadOptions?.ssr;

    if (!realPath.endsWith(".vue")) {
      state.logger.log(`load: skipping non-vue virtual module ${realPath}`);
      return null;
    }
    if (!shouldLoadCompiledVueSfcPath(state, realPath)) {
      return null;
    }
    return loadCompiledSfcModule(state, realPath, isSsr, currentBase, loadOptions);
  }

  if (loadableVueSfcPath) {
    const hasNuxtComponentQuery =
      !!request.querySuffix &&
      new URLSearchParams(request.querySuffix.slice(1)).has("nuxt_component");
    if (!shouldLoadCompiledVueSfcPath(state, loadableVueSfcPath, hasNuxtComponentQuery)) {
      return null;
    }
    const isSsr = !!loadOptions?.ssr;
    return loadCompiledSfcModule(state, loadableVueSfcPath, isSsr, currentBase, loadOptions);
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

function shouldLogSfcWarnings(state: VizePluginState, realPath: string): boolean {
  if (!realPath.includes("node_modules")) return true;
  if ((state.mergedOptions.handleNodeModulesVue ?? true) === false) return false;
  return state.filter(realPath);
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
  // Honor an explicit user exclude (e.g. third-party JSX), but JSX/TSX is not
  // covered by the default `**/*.vue` filter, so a bare extension match is enough.
  if (state.mergedOptions.exclude && !state.filter(request.path)) {
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
  const sourceMap = getCompileOptionsForRequest(state, ssr).sourceMap;

  const {
    code: compiled,
    map,
    warnings,
  } = compileJsxModule(realPath, code, {
    jsxMode: state.mergedOptions.jsxMode,
    vapor: state.mergedOptions.vapor ?? false,
    ssr,
    sourceMap,
  });

  for (const warning of warnings) {
    state.logger.warn(`Warning in ${realPath}: ${warning}`);
  }

  // HMR (deferred, #1533): unlike `.vue` SFCs — whose compiled module exposes a
  // `_sfc_main` component object that the injected `import.meta.hot.accept`
  // boundary attaches an `__hmrId`/HMR record to (see
  // `vize_atelier_sfc::vite_plugin::generate_hmr_code`) — the JSX compiler emits
  // a render-function-only module (`export function render(…)`) with no
  // component object to register against. A state-preserving Vue HMR boundary
  // (`__VUE_HMR_RUNTIME__.rerender`/`reload`) therefore needs the upcoming
  // JSX component-wrapper output before it can hook up; until then `.jsx`/`.tsx`
  // edits fall back to Vite's default module reload. Source map + preamble
  // plumbing (this function's `map`/preamble) land now.
  //
  // Vite's `TransformResult.map` is the object form, so parse the native v3 JSON
  // map before handing it back (#1533).
  return { code: compiled, map: map ? JSON.parse(map) : null };
}

// Strip TypeScript from compiled .vue output and apply define replacements
export async function transformHook(
  state: VizePluginState,
  code: string,
  id: string,
  options?: { ssr?: boolean },
): Promise<TransformResult | null> {
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
  if (request.isVizeVirtual || request.isMacroVirtualId || pluginVisibleVirtualPath) {
    const realPath = request.isMacroVirtualId
      ? (request.strippedVirtualPath ?? "")
      : (request.vizeVirtualPath ?? pluginVisibleVirtualPath ?? "");
    try {
      const result = await transformVirtualTypeScript(code, realPath);
      const defines = getVirtualModuleDefines(state, options?.ssr ?? false);
      let transformed = result.code;
      transformed = applyDefineReplacements(transformed, defines);

      return { code: transformed, map: null };
    } catch (e: unknown) {
      state.logger.error(`transformWithOxc failed for ${realPath}:`, e);
      let dumpPath: string | null = null;
      try {
        dumpPath = getOxcDumpPath(state.root, realPath);
        fs.writeFileSync(dumpPath, code, "utf-8");
        state.logger.error(`Dumped failing code to ${dumpPath}`);
      } catch (dumpError: unknown) {
        state.logger.error(`Failed to dump failing virtual module for ${realPath}:`, dumpError);
      }

      const message = [
        `[vize] Virtual module transform failed for ${realPath}: ${formatUnknownError(e)}`,
        dumpPath ? `Dumped failing code to ${dumpPath}` : null,
      ]
        .filter(Boolean)
        .join("\n");
      throw new Error(message);
    }
  }

  return null;
}

function formatUnknownError(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}
