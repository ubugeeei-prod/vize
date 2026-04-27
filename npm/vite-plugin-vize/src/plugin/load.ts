import fs from "node:fs";
import path from "node:path";
import { pathToFileURL } from "node:url";
import type { TransformResult } from "vite";
import { transformWithOxc } from "vite";

import {
  getCompileOptionsForRequest,
  getEnvironmentCache,
  syncCollectedCssForFile,
  type VizePluginState,
} from "./state.ts";
import { compileFile } from "../compiler.ts";
import { generateOutput, hasDelegatedStyles } from "../utils/index.ts";
import { resolveCssImports, scopeCssForPipeline } from "../utils/css.ts";
import {
  isVizeVirtual,
  isVizeSsrVirtual,
  fromVirtualId,
  LEGACY_VIZE_PREFIX,
  RESOLVED_CSS_MODULE,
  rewriteDynamicTemplateImports,
} from "../virtual.ts";
import { rewriteStaticAssetUrls, applyDefineReplacements } from "../transform.ts";

const SERVER_PLACEHOLDER_CODE = `import { createElementBlock, defineComponent } from "vue";
export default defineComponent({
  name: "ServerPlaceholder",
  render() {
    return createElementBlock("div");
  }
});
`;

export function getBoundaryPlaceholderCode(realPath: string, ssr: boolean): string | null {
  if (ssr && realPath.endsWith(".client.vue")) {
    return SERVER_PLACEHOLDER_CODE;
  }
  if (!ssr && realPath.endsWith(".server.vue")) {
    return SERVER_PLACEHOLDER_CODE;
  }
  return null;
}

function getOxcDumpPath(root: string, realPath: string): string {
  const dumpDir = path.resolve(root || process.cwd(), "__agent_only", "oxc-dumps");
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

function hasQueryParam(id: string, name: string): boolean {
  const query = id.split("?")[1];
  return query ? new URLSearchParams(query).has(name) : false;
}

function hasMacroQuery(id: string): boolean {
  const query = id.split("?")[1];
  return query ? new URLSearchParams(query).get("macro") === "true" : false;
}

function normalizeMacroRealPath(realPath: string): string {
  return realPath.endsWith(".vue.ts") ? realPath.slice(0, -3) : realPath;
}

function stripVirtualQuery(id: string): string {
  return normalizeMacroRealPath(id.slice(1).split("?")[0] ?? "");
}

function findMacroArtifactModule(
  state: VizePluginState,
  realPath: string,
  ssr: boolean,
  kind: string,
): string | null {
  const cache = getEnvironmentCache(state, ssr);
  realPath = normalizeMacroRealPath(realPath);
  let compiled = cache.get(realPath) ?? state.cache.get(realPath) ?? state.ssrCache.get(realPath);

  if (!compiled && fs.existsSync(realPath)) {
    const source = fs.readFileSync(realPath, "utf-8");
    compiled = compileFile(realPath, cache, getCompileOptionsForRequest(state, ssr), source);
    syncCollectedCssForFile(state, realPath, compiled);
  }

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
  loadOptions?: { ssr?: boolean },
): string | { code: string; map: null } | null {
  // Pick the correct viteBase for URL resolution based on the build environment.
  const currentBase = loadOptions?.ssr ? state.serverViteBase : state.clientViteBase;

  // Handle virtual CSS module for production extraction
  if (id === RESOLVED_CSS_MODULE) {
    const allCss = Array.from(state.collectedCss.values()).join("\n\n");
    return allCss;
  }

  // Strip the \0 prefix and the appended extension suffix for style virtual IDs.
  let styleId = id;
  if (id.startsWith("\0") && id.includes("?vue")) {
    styleId = id
      .slice(1) // strip \0
      .replace(/\.module\.\w+$/, "") // strip .module.{lang}
      .replace(/\.\w+$/, ""); // strip .{lang}
  }

  if (styleId.includes("?vue&type=style") || styleId.includes("?vue=&type=style")) {
    const [filename, queryString] = styleId.split("?");
    const realPath = isVizeVirtual(filename) ? fromVirtualId(filename) : filename;
    const params = new URLSearchParams(queryString);
    const indexStr = params.get("index");
    const lang = params.get("lang");
    const _hasModule = params.has("module");
    const scoped = params.get("scoped");

    const compiled = state.cache.get(realPath);
    const fallbackCompiled = compiled ?? state.ssrCache.get(realPath);
    const blockIndex = indexStr !== null ? parseInt(indexStr, 10) : -1;

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

      // For scoped preprocessor styles, wrap content in a scope selector
      if (scoped && block.scoped && lang && lang !== "css") {
        const lines = styleContent.split("\n");
        const hoisted: string[] = [];
        const body: string[] = [];
        for (const line of lines) {
          const trimmed = line.trimStart();
          if (
            trimmed.startsWith("@use ") ||
            trimmed.startsWith("@forward ") ||
            trimmed.startsWith("@import ")
          ) {
            hoisted.push(line);
          } else {
            body.push(line);
          }
        }
        const bodyContent = body.join("\n");
        const hoistedContent = hoisted.length > 0 ? hoisted.join("\n") + "\n\n" : "";
        styleContent = `${hoistedContent}[${scoped}] {\n${bodyContent}\n}`;
      }

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
    return "";
  }

  // Handle Vue Router's ?definePage query through extracted artifacts.
  if (id.startsWith("\0") && hasQueryParam(id, "definePage")) {
    return loadDefinePageArtifact(state, stripVirtualQuery(id), !!loadOptions?.ssr);
  }

  // Handle ?macro=true queries
  if (id.startsWith("\0") && hasMacroQuery(id)) {
    const realPath = stripVirtualQuery(id);
    const artifactLoad = loadDefinePageMetaArtifact(state, realPath, !!loadOptions?.ssr);
    if (artifactLoad) {
      return artifactLoad;
    }

    if (fs.existsSync(realPath)) {
      const source = fs.readFileSync(realPath, "utf-8");
      const setupMatch = source.match(/<script\s+setup[^>]*>([\s\S]*?)<\/script>/);
      if (setupMatch) {
        const scriptContent = setupMatch[1];
        return {
          code: `${scriptContent}\nexport default {}`,
          map: null,
        };
      }
    }
    return { code: "export default {}", map: null };
  }

  // Handle vize virtual modules
  if (isVizeVirtual(id)) {
    const realPath = fromVirtualId(id);
    const isSsr = isVizeSsrVirtual(id) || !!loadOptions?.ssr;

    if (!realPath.endsWith(".vue")) {
      state.logger.log(`load: skipping non-vue virtual module ${realPath}`);
      return null;
    }

    const placeholderCode = getBoundaryPlaceholderCode(realPath, !!loadOptions?.ssr);
    if (placeholderCode) {
      state.logger.log(`load: using boundary placeholder for ${realPath}`);
      return {
        code: placeholderCode,
        map: null,
      };
    }

    const cache = getEnvironmentCache(state, isSsr);
    let compiled = cache.get(realPath);

    // On-demand compile if not cached
    if (!compiled && fs.existsSync(realPath)) {
      state.logger.log(`load: on-demand compiling ${realPath}`);
      compiled = compileFile(realPath, cache, getCompileOptionsForRequest(state, isSsr));
      syncCollectedCssForFile(state, realPath, compiled);
    }

    if (compiled) {
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
      const output = rewriteStaticAssetUrls(
        rewriteDynamicTemplateImports(
          generateOutput(compiled, {
            isProduction: state.isProduction,
            isDev: state.server !== null && !isSsr,
            ssr: isSsr,
            hmrUpdateType: pendingHmrUpdateType,
            extractCss: state.extractCss,
            filePath: realPath,
          }),
          state.dynamicImportAliasRules,
        ),
        state.dynamicImportAliasRules,
      );
      if (!loadOptions?.ssr) {
        state.pendingHmrUpdateTypes.delete(realPath);
      }
      return {
        code: output,
        map: null,
      };
    }
  }

  // Handle \0-prefixed non-vue files leaked from virtual module dynamic imports.
  if (id.startsWith("\0")) {
    const afterPrefix = id.startsWith(LEGACY_VIZE_PREFIX)
      ? id.slice(LEGACY_VIZE_PREFIX.length)
      : id.slice(1);
    if (afterPrefix.includes("?commonjs-")) {
      return null;
    }
    const [pathPart, queryPart] = afterPrefix.split("?");
    const querySuffix = queryPart ? `?${queryPart}` : "";
    const fsPath = pathPart.startsWith("/@fs/") ? pathPart.slice(4) : pathPart;
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

// Strip TypeScript from compiled .vue output and apply define replacements
export async function transformHook(
  state: VizePluginState,
  code: string,
  id: string,
  options?: { ssr?: boolean },
): Promise<TransformResult | null> {
  const isMacro = id.startsWith("\0") && (hasMacroQuery(id) || hasQueryParam(id, "definePage"));
  if (isVizeVirtual(id) || isMacro) {
    const realPath = isMacro ? stripVirtualQuery(id) : fromVirtualId(id);
    try {
      const result = await transformWithOxc(code, realPath, {
        lang: "ts",
      });
      const defines = getVirtualModuleDefines(state, options?.ssr ?? false);
      let transformed = result.code;
      transformed = applyDefineReplacements(transformed, defines);

      return { code: transformed, map: result.map as TransformResult["map"] };
    } catch (e: unknown) {
      state.logger.error(`transformWithOxc failed for ${realPath}:`, e);
      const dumpPath = getOxcDumpPath(state.root, realPath);
      fs.writeFileSync(dumpPath, code, "utf-8");
      state.logger.error(`Dumped failing code to ${dumpPath}`);
      return { code: "export default {}", map: null };
    }
  }

  return null;
}
