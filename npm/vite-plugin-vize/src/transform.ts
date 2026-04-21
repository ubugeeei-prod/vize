/**
 * Code transformation utilities for Vize.
 *
 * Handles static asset URL rewriting, Vite define replacements, and
 * provides the debug logger.
 */

import { escapeRegExp, type DynamicImportAliasRule } from "./virtual.ts";

/**
 * Rewrite static asset URLs in compiled template output.
 *
 * Transforms property values like `src: "@/assets/logo.svg"` into import
 * statements hoisted to the top of the module, so Vite's module resolution
 * pipeline handles alias expansion and asset hashing in both dev and build.
 */
// File extensions that are code modules, not static assets.
// These should never be rewritten to default imports by rewriteStaticAssetUrls.
const SCRIPT_EXTENSIONS = /\.(js|mjs|cjs|ts|mts|cts|jsx|tsx)$/i;

export function rewriteStaticAssetUrls(code: string, aliasRules: DynamicImportAliasRule[]): string {
  let rewritten = code;
  const imports: string[] = [];
  let counter = 0;

  for (const rule of aliasRules) {
    // Match patterns:
    //   src: "@/..."  or  "src": "@/..."  (double quotes)
    //   src: '@/...'  or  "src": '@/...'  (single quotes)
    const pattern = new RegExp(
      `("?src"?\\s*:\\s*)(?:"(${escapeRegExp(rule.fromPrefix)}[^"]+)"|'(${escapeRegExp(rule.fromPrefix)}[^']+)')`,
      "g",
    );
    rewritten = rewritten.replace(
      pattern,
      (match: string, prefix: string, dqPath?: string, sqPath?: string) => {
        const fullPath = dqPath || sqPath;
        // Skip script files -- they are code modules, not static assets.
        if (fullPath && SCRIPT_EXTENSIONS.test(fullPath)) {
          return match;
        }
        const varName = `__vize_static_${counter++}`;
        imports.push(`import ${varName} from ${JSON.stringify(fullPath)};`);
        return `${prefix}${varName}`;
      },
    );
  }

  if (imports.length > 0) {
    rewritten = imports.join("\n") + "\n" + rewritten;
  }
  return rewritten;
}

/**
 * Built-in Vite/Vue/Nuxt define keys that are handled by Vite's own transform pipeline.
 * These must NOT be replaced by the vize plugin because:
 * 1. Nuxt runs both client and server Vite builds, each with different values
 *    (e.g., import.meta.server = true on server, false on client).
 * 2. Vite's import.meta transform already handles these correctly per-environment.
 */
const BUILTIN_DEFINE_PREFIXES = [
  "import.meta.server",
  "import.meta.client",
  "import.meta.dev",
  "import.meta.test",
  "import.meta.prerender",
  "import.meta.env",
  "import.meta.hot",
  "__VUE_",
  "__NUXT_",
  "process.env",
];

export function isBuiltinDefine(key: string): boolean {
  return BUILTIN_DEFINE_PREFIXES.some(
    (prefix) => key === prefix || key.startsWith(prefix + ".") || key.startsWith(prefix + "_"),
  );
}

/**
 * Apply Vite define replacements to code.
 * Replaces keys like `import.meta.vfFeatures.photoSection` with their values.
 * Uses word-boundary-aware matching to avoid replacing inside strings or partial matches.
 */
export function applyDefineReplacements(code: string, defines: Record<string, string>): string {
  // Sort keys longest-first to prevent partial matches (e.g., "import.meta.env" before "import.meta")
  const sortedKeys = Object.keys(defines).sort((a, b) => b.length - a.length);
  let result = code;
  for (const key of sortedKeys) {
    if (!result.includes(key)) continue;
    // Build a regex that matches the key not preceded/followed by word chars or dots
    // This prevents matching inside strings or longer identifiers
    const escaped = key.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
    const re = new RegExp(escaped + "(?![\\w$.])", "g");
    result = result.replace(re, defines[key]);
  }
  return result;
}

export function createLogger(debug: boolean) {
  return {
    log: (...args: unknown[]) => debug && console.log("[vize]", ...args),
    info: (...args: unknown[]) => console.log("[vize]", ...args),
    warn: (...args: unknown[]) => console.warn("[vize]", ...args),
    error: (...args: unknown[]) => console.error("[vize]", ...args),
  };
}
