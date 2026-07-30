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

/**
 * String pre-gate for {@link fromPluginVisibleVirtualId}, so ordinary module IDs
 * never cross the native boundary (#3427).
 *
 * A non-null result requires `request.path` to end with `.vue.ts` or `.vue.tsx`
 * and `request.querySuffix` to be non-empty. `request.path` is `id` up to the
 * first `?` and `querySuffix` is non-empty exactly when that `?` exists, so both
 * conditions imply these two substring tests. The tests are strictly weaker, so
 * everything the old code accepted still reaches the classifier.
 */
function mayBePluginVisibleVirtualId(id: string): boolean {
  return !id.startsWith("\0") && id.includes(".vue.ts") && id.includes("?");
}

export function fromPluginVisibleVirtualId(id: string): string | null {
  if (!mayBePluginVisibleVirtualId(id)) {
    return null;
  }
  const request = classifyVitePluginRequest(id);
  if (!isPluginVisibleVueVirtualPath(request.path) || !request.querySuffix) {
    return null;
  }
  const params = new URLSearchParams(request.querySuffix.slice(1));
  if (!params.has("vue") || (!params.has("vize") && !params.has("vize-ssr"))) {
    return null;
  }
  return stripPluginVisibleVueVirtualSuffix(stripFsPrefix(request.path));
}

/**
 * The `path` a second `classifyVitePluginRequest(request.normalizedFsId ?? id)`
 * used to recompute (#3427).
 *
 * `normalizedFsId` is `Some` exactly when the pre-`?` path starts with `/@fs`,
 * and its value is that path with the four-byte prefix removed plus the original
 * query suffix. Re-splitting that at the first `?` therefore yields the path
 * without the prefix — and when `normalizedFsId` is `undefined` the second call
 * classified `id` itself and yielded `request.path` unchanged.
 */
function stripFsPrefix(path: string): string {
  return path.startsWith("/@fs") ? path.slice(4) : path;
}

function isPluginVisibleVueVirtualPath(path: string): boolean {
  return path.endsWith(".vue.ts") || path.endsWith(".vue.tsx");
}

function stripPluginVisibleVueVirtualSuffix(path: string): string {
  if (path.endsWith(".vue.tsx")) {
    return path.slice(0, -4);
  }
  return path.endsWith(".vue.ts") ? path.slice(0, -3) : path;
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
