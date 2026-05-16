/**
 * Code transformation utilities for Vize.
 *
 * Handles static asset URL rewriting, Vite define replacements, and
 * provides the debug logger.
 */

import {
  applyViteDefineReplacements,
  isBuiltinViteDefine,
  rewriteViteStaticAssetUrls,
  shouldApplyViteDefineInVirtualModule,
} from "@vizejs/native";
import type { DynamicImportAliasRule } from "./virtual.ts";

/**
 * Rewrite static asset URLs in compiled template output.
 *
 * Transforms property values like `src: "@/assets/logo.svg"` into import
 * statements hoisted to the top of the module, so Vite's module resolution
 * pipeline handles alias expansion and asset hashing in both dev and build.
 */
export function rewriteStaticAssetUrls(code: string, aliasRules: DynamicImportAliasRule[]): string {
  return rewriteViteStaticAssetUrls(code, aliasRules);
}

/**
 * Built-in Vite/Vue/Nuxt define keys that are normally handled by Vite's own
 * transform pipeline.
 */
export function isBuiltinDefine(key: string): boolean {
  return isBuiltinViteDefine(key);
}

export function shouldApplyDefineInVirtualModule(key: string): boolean {
  return shouldApplyViteDefineInVirtualModule(key);
}

/**
 * Apply Vite define replacements to code.
 * Replaces keys like `import.meta.vfFeatures.photoSection` with their values.
 * Uses word-boundary-aware matching to avoid replacing inside strings or partial matches.
 */
export function applyDefineReplacements(code: string, defines: Record<string, string>): string {
  return applyViteDefineReplacements(
    code,
    Object.entries(defines).map(([key, value]) => ({ key, value })),
  );
}

export function createLogger(debug: boolean) {
  return {
    log: (...args: unknown[]) => debug && console.log("[vize]", ...args),
    info: (...args: unknown[]) => console.log("[vize]", ...args),
    warn: (...args: unknown[]) => console.warn("[vize]", ...args),
    error: (...args: unknown[]) => console.error("[vize]", ...args),
  };
}
