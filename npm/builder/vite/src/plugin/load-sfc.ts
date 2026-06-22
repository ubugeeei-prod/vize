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

export function shouldLoadCompiledVueSfcPath(state: VizePluginState, realPath: string): boolean {
  const isNodeModulesPath = realPath.includes("node_modules");
  const handleNodeModules = state.mergedOptions.handleNodeModulesVue ?? true;

  if (!handleNodeModules && isNodeModulesPath) {
    state.logger.log(`load: skipping node_modules Vue SFC ${realPath}`);
    return false;
  }

  if (!isNodeModulesPath && !state.filter(realPath)) {
    state.logger.log(`load: skipping filtered Vue SFC ${realPath}`);
    return false;
  }

  return true;
}
