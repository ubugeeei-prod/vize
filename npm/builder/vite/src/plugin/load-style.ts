import fs from "node:fs";
import { compileFile } from "../compiler.ts";
import { getCompileOptionsForRequest, getEnvironmentCache, type VizePluginState } from "./state.ts";
import { shouldLoadCompiledVueSfcPath } from "./load-sfc.ts";
import type { StyleBlockInfo } from "../sfc-types.ts";
import { scopeCssForPipeline, transformCssVarsForPipeline } from "../utils/css.ts";

export { normalizeStyleVirtualId } from "./style-request.ts";

/** Nuxt can load an emitted CSS entry before loading its owning SFC. */
export function getCompiledStyleSource(state: VizePluginState, realPath: string, ssr: boolean) {
  const cache = getEnvironmentCache(state, ssr);
  const compiled = cache.get(realPath) ?? state.cache.get(realPath) ?? state.ssrCache.get(realPath);
  if (compiled || !shouldLoadCompiledVueSfcPath(state, realPath) || !fs.existsSync(realPath)) {
    return compiled;
  }
  return compileFile(realPath, cache, getCompileOptionsForRequest(state, ssr, realPath));
}

export interface VirtualStyleOptions {
  scopeId: string;
  scoped: string | null;
  lang: string | null;
  trim?: boolean;
  productionFilename?: string;
}

export function getVirtualStyleContent(
  block: StyleBlockInfo,
  options: VirtualStyleOptions,
): string {
  // Virtual style blocks retain source whitespace; apply the plugin-vue
  // trim option before handing their contents to Vite's CSS pipeline.
  let styleContent = options.trim === false ? block.content : block.content.trim();

  // Keep delegated plain CSS scoped while preserving PostCSS-only syntax
  // such as `@apply` for the downstream CSS pipeline.
  if (options.scoped && block.scoped && (!options.lang || options.lang === "css")) {
    styleContent = scopeCssForPipeline(styleContent, options.scoped);
  }

  return transformCssVarsForPipeline(styleContent, options.scopeId, options.productionFilename);
}
