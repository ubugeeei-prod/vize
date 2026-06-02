/**
 * Virtual module ID management and dynamic import rewriting for Vize.
 *
 * Handles the mapping between real .vue file paths and their virtual module
 * counterparts, as well as rewriting dynamic template imports for alias resolution.
 */

import {
  classifyVitePluginRequest,
  createViteVirtualId,
  fromViteVirtualId,
  normalizeViteFsIdForBuild,
  normalizeViteVirtualVueModuleId as normalizeNativeViteVirtualVueModuleId,
  rewriteViteDynamicTemplateImports,
  toViteBrowserImportPrefix,
} from "@vizejs/native";

// Virtual module prefixes and constants
export const LEGACY_VIZE_PREFIX = "\0vize:";
export const VIZE_SSR_PREFIX = "\0vize-ssr:";
export const VIRTUAL_CSS_MODULE = "virtual:vize-styles";
export const RESOLVED_CSS_MODULE = "\0vize:all-styles.css";

export interface DynamicImportAliasRule {
  fromPrefix: string;
  toPrefix: string;
}

/** Check if a module ID is a vize-compiled virtual module */
export function isVizeVirtual(id: string): boolean {
  return classifyVitePluginRequest(id).isVizeVirtual;
}

export function isVizeVirtualVueModuleId(id: string): boolean {
  return classifyVitePluginRequest(id).isVizeVirtual;
}

export function isVizeSsrVirtual(id: string): boolean {
  return classifyVitePluginRequest(id).isVizeSsrVirtual;
}

/** Create a virtual module ID from a real .vue file path */
export function toVirtualId(realPath: string, ssr = false): string {
  return createViteVirtualId(realPath, ssr);
}

export function toPluginVisibleVirtualId(realPath: string, ssr = false, querySuffix = ""): string {
  const params = new URLSearchParams(querySuffix.startsWith("?") ? querySuffix.slice(1) : "");
  params.delete("vue");
  params.delete("vize");
  params.delete("vize-ssr");
  const rest = params.toString();
  return `${realPath}.ts?vue&${ssr ? "vize-ssr" : "vize"}${rest ? `&${rest}` : ""}`;
}

export function fromPluginVisibleVirtualId(id: string): string | null {
  if (id.startsWith("\0")) {
    return null;
  }
  const request = classifyVitePluginRequest(id);
  if (!request.path.endsWith(".vue.ts") || !request.querySuffix) {
    return null;
  }
  const params = new URLSearchParams(request.querySuffix.slice(1));
  if (!params.has("vue") || (!params.has("vize") && !params.has("vize-ssr"))) {
    return null;
  }
  return classifyVitePluginRequest(request.normalizedFsId ?? id).normalizedVuePath;
}

export function isPluginVisibleSsrVirtualId(id: string): boolean {
  const request = classifyVitePluginRequest(id);
  return request.querySuffix
    ? new URLSearchParams(request.querySuffix.slice(1)).has("vize-ssr")
    : false;
}

/** Extract the real .vue file path from a virtual module ID */
export function fromVirtualId(virtualId: string): string {
  return fromViteVirtualId(virtualId);
}

export function normalizeVizeVirtualVueModuleId(id: string): string {
  return normalizeNativeViteVirtualVueModuleId(id);
}

export function toBrowserImportPrefix(replacement: string): string {
  return toViteBrowserImportPrefix(replacement);
}

export function normalizeFsIdForBuild(id: string): string {
  return normalizeViteFsIdForBuild(id);
}

export function rewriteDynamicTemplateImports(
  code: string,
  aliasRules: DynamicImportAliasRule[],
): string {
  return rewriteViteDynamicTemplateImports(code, aliasRules);
}
