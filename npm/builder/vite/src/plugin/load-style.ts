import fs from "node:fs";
import { compileFile } from "../compiler.ts";
import { getCompileOptionsForRequest, getEnvironmentCache, type VizePluginState } from "./state.ts";
import { shouldLoadCompiledVueSfcPath } from "./load-sfc.ts";

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
