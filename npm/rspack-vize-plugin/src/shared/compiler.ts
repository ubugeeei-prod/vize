/**
 * Core compilation logic for @vizejs/rspack-plugin
 * Copied and adapted from vite-plugin-vize/src/compiler.ts
 */

import { createHash } from "node:crypto";
import * as native from "@vizejs/native";
import type { CompiledModule, SfcCompileOptionsNapi } from "../types/index.js";
import { generateScopeId, extractStyleBlocks } from "./utils.js";

const { compileSfc } = native;

// ============================================================================
// Compilation Cache
// ============================================================================

interface CacheEntry {
  contentHash: string;
  result: CompiledModule;
}

/**
 * Module-level cache to avoid re-compiling unchanged files across loader runs.
 * Key: filePath, Value: { contentHash, result }.
 * In watch mode Rspack re-invokes the loader for changed files, but unchanged
 * files that are re-evaluated (e.g. due to dependency changes) will hit the cache.
 */
const compilationCache = new Map<string, CacheEntry>();

function computeContentHash(source: string): string {
  return createHash("sha256").update(source).digest("hex").slice(0, 16);
}

/**
 * Clear the compilation cache.  Exposed for testing and manual invalidation.
 */
export function clearCompilationCache(): void {
  compilationCache.clear();
}

/**
 * Compile a single .vue file.
 *
 * Adapted from vite-plugin-vize for Rspack loader scenario:
 * - Uses content-hash based cache to skip re-compilation of unchanged files
 * - Does not read file (source is passed as parameter)
 * - Returns styles metadata for loader chain processing
 */
export function compileFile(
  filePath: string,
  source: string,
  options: {
    sourceMap?: boolean;
    ssr?: boolean;
    compilerOptions?: SfcCompileOptionsNapi;
  } = {},
): CompiledModule {
  // Check content-hash cache to skip re-compilation of unchanged files
  const contentHash = computeContentHash(source);
  const cached = compilationCache.get(filePath);
  if (cached && cached.contentHash === contentHash) {
    return cached.result;
  }

  const scopeId = generateScopeId(filePath);
  const hasScoped = /<style[^>]*\bscoped\b/.test(source);

  // Auto-detect TypeScript from <script lang="ts"> or <script setup lang="ts">
  const autoIsTs =
    options.compilerOptions?.isTs ??
    /<script[^>]*\blang=["']ts["']/.test(source);

  const napiOptions: SfcCompileOptionsNapi = {
    ...options.compilerOptions,
    filename: filePath,
    sourceMap: options.sourceMap ?? options.compilerOptions?.sourceMap ?? true,
    ssr: options.ssr ?? options.compilerOptions?.ssr ?? false,
    isTs: autoIsTs,
    scopeId: hasScoped ? `data-v-${scopeId}` : undefined,
  };

  const result = compileSfc(source, napiOptions);

  const styles = extractStyleBlocks(source);

  const compiled: CompiledModule = {
    code: result.code,
    css: result.css,
    errors: result.errors,
    warnings: result.warnings,
    scopeId,
    hasScoped,
    styles,
  };

  // Only cache successful compilations (no errors)
  if (compiled.errors.length === 0) {
    compilationCache.set(filePath, { contentHash, result: compiled });
  }

  return compiled;
}

/**
 * Generate output code with style imports injected.
 *
 * Rspack version: Does not inject HMR code (handled by Rspack watch mode).
 *
 * Key difference from Vite version:
 * - Generates import statements with query parameters for style blocks
 * - Rspack will route these to the appropriate style loader via resourceQuery matching
 */
export function generateOutput(
  compiled: CompiledModule,
  options: {
    requestPath: string;
  },
): string {
  let output = compiled.code;

  // Handle export default transformation
  const exportDefaultRegex = /^export default /m;
  const hasExportDefault = exportDefaultRegex.test(output);
  const hasSfcMainDefined = /\bconst\s+_sfc_main\s*=/.test(output);

  if (hasExportDefault && !hasSfcMainDefined) {
    output = output.replace(exportDefaultRegex, "const _sfc_main = ");
    // Add __scopeId for scoped CSS support
    if (compiled.hasScoped && compiled.scopeId) {
      output += `\n_sfc_main.__scopeId = "data-v-${compiled.scopeId}";`;
    }
    output += "\nexport default _sfc_main;";
  } else if (hasExportDefault && hasSfcMainDefined) {
    // _sfc_main already defined, just add scopeId if needed
    if (compiled.hasScoped && compiled.scopeId) {
      output = output.replace(
        /^export default _sfc_main/m,
        `_sfc_main.__scopeId = "data-v-${compiled.scopeId}";\nexport default _sfc_main`,
      );
    }
  }

  // Inject style imports (key difference: using query parameters for Rspack loader chain)
  if (compiled.styles.length > 0) {
    const styleImports = compiled.styles
      .map((style, index) => {
        // Build query string manually (not URLSearchParams) to match vue-loader convention:
        // ?vue&type=style&index=0&lang=css  (NOT ?vue=true&type=style&...)
        const queryParts = [
          "vue",
          "type=style",
          `index=${index}`,
          `lang=${style.lang || "css"}`,
          ...(style.scoped ? [`scoped=${compiled.scopeId}`] : []),
          ...(style.module
            ? [`module=${typeof style.module === "string" ? style.module : "true"}`]
            : []),
        ];
        const queryStr = queryParts.join("&");

        // CSS Modules require default import
        if (style.module) {
          const varName = typeof style.module === "string" ? style.module : "$style";
          return `import ${varName} from ${JSON.stringify(`${options.requestPath}?${queryStr}`)};`;
        }
        return `import ${JSON.stringify(`${options.requestPath}?${queryStr}`)};`;
      })
      .join("\n");

    output = styleImports + "\n" + output;

    // Add CSS module bindings to component
    const cssModuleBlocks = compiled.styles.filter((s) => s.module);
    if (cssModuleBlocks.length > 0) {
      const moduleBindings: { name: string; bindingName: string }[] = [];
      for (const block of cssModuleBlocks) {
        const bindingName = typeof block.module === "string" ? block.module : "$style";
        moduleBindings.push({ name: bindingName, bindingName });
      }

      const cssModuleSetup = moduleBindings
        .map(
          (m) =>
            `_sfc_main.__cssModules = _sfc_main.__cssModules || {};\n_sfc_main.__cssModules[${JSON.stringify(m.name)}] = ${m.bindingName};`,
        )
        .join("\n");

      // Insert before the final "export default _sfc_main;"
      output = output.replace(
        /^export default _sfc_main;/m,
        `${cssModuleSetup}\nexport default _sfc_main;`,
      );
    }
  }

  return output;
}
