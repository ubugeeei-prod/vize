import { classifyVitePluginRequest } from "@vizejs/native";
import type { VizePluginState } from "./state.ts";

function shouldLoadVueSfcRequest(request: ReturnType<typeof classifyVitePluginRequest>): boolean {
  if (
    !request.isVueSfcPath ||
    request.isVueStyleQuery ||
    request.hasMacroQuery ||
    request.hasDefinePageQuery
  ) {
    return false;
  }

  if (!request.querySuffix) {
    return true;
  }

  const params = new URLSearchParams(request.querySuffix.slice(1));
  if (
    params.has("raw") ||
    params.has("url") ||
    params.has("worker") ||
    params.has("sharedworker")
  ) {
    return false;
  }

  return params.has("nuxt_component");
}

export function getLoadableVueSfcPath(
  request: ReturnType<typeof classifyVitePluginRequest>,
): string | null {
  if (!shouldLoadVueSfcRequest(request)) {
    return null;
  }
  return classifyVitePluginRequest(request.normalizedFsId ?? request.path).normalizedVuePath;
}

export function shouldLoadCompiledVueSfcPath(
  state: VizePluginState,
  realPath: string,
  hasNuxtComponentQuery = false,
): boolean {
  const isNodeModulesPath = realPath.includes("node_modules");
  const handleNodeModules = state.mergedOptions.handleNodeModulesVue ?? true;

  // Only skip node_modules Vue SFCs for runtime ?nuxt_component loads.
  // During production builds, node_modules .vue files must still be compiled
  // by Vize because the Vite/Rollup transform pipeline (e.g. vite:define) may
  // run before plugin-vue and cannot parse raw Vue SFC syntax.
  if (!handleNodeModules && isNodeModulesPath && hasNuxtComponentQuery) {
    state.logger.log(`load: skipping node_modules Vue SFC ${realPath}`);
    return false;
  }

  if (!isNodeModulesPath && !state.filter(realPath)) {
    state.logger.log(`load: skipping filtered Vue SFC ${realPath}`);
    return false;
  }

  return true;
}
