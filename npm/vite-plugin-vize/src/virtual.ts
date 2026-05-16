/**
 * Virtual module ID management and dynamic import rewriting for Vize.
 *
 * Handles the mapping between real .vue file paths and their virtual module
 * counterparts, as well as rewriting dynamic template imports for alias resolution.
 */

import path from "node:path";
import fs from "node:fs";
import { classifyVitePluginRequest } from "@vizejs/native";

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
  return ssr ? `${VIZE_SSR_PREFIX}${realPath}.ts` : "\0" + realPath + ".ts";
}

/** Extract the real .vue file path from a virtual module ID */
export function fromVirtualId(virtualId: string): string {
  const request = classifyVitePluginRequest(virtualId);
  if (request.vizeVirtualPath) {
    return request.vizeVirtualPath;
  }
  const normalized = normalizeVizeVirtualVueModuleId(virtualId);
  const queryStart = normalized.indexOf("?");
  return queryStart === -1 ? normalized : normalized.slice(0, queryStart);
}

export function normalizeVizeVirtualVueModuleId(id: string): string {
  const request = classifyVitePluginRequest(id);
  if (request.vizeVirtualPath) {
    return request.vizeVirtualPath + request.querySuffix;
  }
  return request.normalizedVuePath + request.querySuffix;
}

export function escapeRegExp(value: string): string {
  return value.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}

export function toBrowserImportPrefix(replacement: string): string {
  const normalized = replacement.replace(/\\/g, "/");
  if (normalized.startsWith("/@fs/")) {
    return normalized;
  }
  // Absolute filesystem alias targets should be served via /@fs in browser imports.
  if (path.isAbsolute(replacement) && fs.existsSync(replacement)) {
    return `/@fs${normalized}`;
  }
  return normalized;
}

export function normalizeFsIdForBuild(id: string): string {
  const [pathPart, queryPart] = id.split("?");
  if (!pathPart.startsWith("/@fs/")) {
    return id;
  }
  const normalizedPath = pathPart.slice(4); // strip '/@fs'
  return queryPart ? `${normalizedPath}?${queryPart}` : normalizedPath;
}

export function rewriteDynamicTemplateImports(
  code: string,
  aliasRules: DynamicImportAliasRule[],
): string {
  let rewritten = code;

  // Normalize alias-based template literal imports (e.g. `@/foo/${x}.svg`) to browser paths.
  for (const rule of aliasRules) {
    const pattern = new RegExp(`\\bimport\\s*\\(\\s*\`${escapeRegExp(rule.fromPrefix)}`, "g");
    rewritten = rewritten.replace(pattern, `import(/* @vite-ignore */ \`${rule.toPrefix}`);
  }

  // Dynamic template imports are intentionally runtime-resolved: mark them to silence
  // Vite's static analysis warning while keeping runtime behavior.
  rewritten = rewritten.replace(/\bimport\s*\(\s*`/g, "import(/* @vite-ignore */ `");

  return rewritten;
}
