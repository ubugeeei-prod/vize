/**
 * Core compilation logic for @vizejs/rspack-plugin
 * Copied and adapted from vite-plugin-vize/src/compiler.ts
 */

import { createHash } from "node:crypto";
import path from "node:path";
import * as native from "@vizejs/native";
import type { CompiledModule, SfcCompileOptionsNapi } from "../types/index.js";
import { generateScopeId, extractStyleBlocks, extractCustomBlocks } from "./utils.js";
import { genHotReloadCode, genCSSModuleHotReloadCode } from "./hotReload.js";

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
 * Key: composite of filePath + compilation options (ssr, isTs), Value: { contentHash, result }.
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
    isCustomElement?: boolean;
    rootContext?: string;
    isProduction?: boolean;
  } = {},
): CompiledModule {
  // Auto-detect TypeScript from <script lang="ts"> or <script setup lang="ts">
  const autoIsTs = options.compilerOptions?.isTs ?? /<script[^>]*\blang=["']ts["']/.test(source);

  // Build a composite cache key that includes compilation-affecting options.
  // Without this, the same file compiled with different ssr/isTs/sourceMap flags
  // would return a stale cached result (e.g. client build reusing SSR output).
  const ssr = options.ssr ?? options.compilerOptions?.ssr ?? false;
  const sourceMap = options.sourceMap ?? options.compilerOptions?.sourceMap ?? true;
  const isCustomElement = options.isCustomElement ?? false;
  const rootCtx = options.rootContext ?? "";
  const isProd = options.isProduction ?? false;
  const cacheKey = `${filePath}:ssr=${ssr}:ts=${autoIsTs}:map=${sourceMap}:ce=${isCustomElement}:root=${rootCtx}:prod=${isProd}`;

  // Check content-hash cache to skip re-compilation of unchanged files
  const contentHash = computeContentHash(source);
  const cached = compilationCache.get(cacheKey);
  if (cached && cached.contentHash === contentHash) {
    return cached.result;
  }

  const scopeId = generateScopeId(filePath, options.rootContext, options.isProduction, source);
  const hasScoped = /<style[^>]*\bscoped\b/.test(source);

  const napiOptions: SfcCompileOptionsNapi = {
    ...options.compilerOptions,
    filename: filePath,
    sourceMap: options.sourceMap ?? options.compilerOptions?.sourceMap ?? true,
    ssr,
    isTs: autoIsTs,
    scopeId: hasScoped ? `data-v-${scopeId}` : undefined,
  };

  const result = compileSfc(source, napiOptions);

  const styles = extractStyleBlocks(source);
  const customBlocks = extractCustomBlocks(source);

  const compiled: CompiledModule = {
    code: result.code,
    css: result.css,
    errors: result.errors,
    warnings: result.warnings,
    scopeId,
    hasScoped,
    styles,
    customBlocks,
    isCustomElement,
  };

  // Only cache successful compilations (no errors)
  if (compiled.errors.length === 0) {
    compilationCache.set(cacheKey, { contentHash, result: compiled });
  }

  return compiled;
}

/**
 * Generate output code with style imports and custom block imports injected.
 *
 * Key difference from Vite version:
 * - Generates import statements with query parameters for style blocks
 * - Rspack will route these to the appropriate style loader via resourceQuery matching
 * - Injects `module.hot` based HMR code when `hmr` is enabled
 */
export function generateOutput(
  compiled: CompiledModule,
  options: {
    requestPath: string;
    /** Inject HMR boilerplate using `module.hot` (Rspack/webpack CJS API) */
    hmr?: boolean;
    /** Original file path (for __file exposure in dev mode) */
    filePath?: string;
    /** Whether this is a production build */
    isProduction?: boolean;
    /** Project root context (for computing relative __file path) */
    rootContext?: string;
  },
): string {
  let output = compiled.code;
  const isCustomElement = compiled.isCustomElement;

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
    if (isCustomElement) {
      // Custom element mode: <style module> is not supported
      const hasModule = compiled.styles.some((s) => s.module);
      if (hasModule) {
        throw new Error(`[vize] <style module> is not supported in custom elements mode.`);
      }
    }

    // Validate: Vue SFC spec allows at most one unnamed <style module>.
    // Multiple unnamed modules would generate duplicate `import $style` bindings,
    // producing invalid ESM code.
    const unnamedModuleCount = compiled.styles.filter((s) => s.module === true).length;
    if (unnamedModuleCount > 1) {
      throw new Error(
        `[vize] Found ${unnamedModuleCount} unnamed <style module> blocks. ` +
          `Only one unnamed <style module> is allowed per SFC. ` +
          `Use named modules instead: <style module="name">`,
      );
    }

    // Filter out empty style blocks that have no content and no external src
    const activeStyles = compiled.styles.filter((style) => style.src || /\S/.test(style.content));

    // Track CSS Module requests for HMR and __cssModules binding.
    // varName: safe JS identifier used in import statement (e.g. _cssModule_0)
    // bindingName: original module name from <style module="..."> (may not be a valid identifier)
    const cssModuleHmrEntries: { request: string; varName: string; bindingName: string }[] = [];

    const styleImports = activeStyles
      .map((style) => {
        const queryParts = [
          "vue",
          "type=style",
          `index=${style.index}`,
          `lang=${style.lang || "css"}`,
          ...(style.scoped ? [`scoped=${compiled.scopeId}`] : []),
          ...(style.module
            ? [`module=${typeof style.module === "string" ? style.module : "true"}`]
            : []),
          ...(isCustomElement ? ["inline"] : []),
        ];
        const queryStr = queryParts.join("&");
        const request = `${options.requestPath}?${queryStr}`;

        if (isCustomElement) {
          return `import _style_${style.index} from ${JSON.stringify(request)};`;
        }

        if (style.module) {
          const bindingName = typeof style.module === "string" ? style.module : "$style";
          // Always use a safe internal variable name for the import binding.
          // The original module name (e.g. "foo-bar") may not be a valid JS
          // identifier, so we use _cssModule_<index> and map it back via
          // __cssModules[bindingName] below.
          const varName = `_cssModule_${style.index}`;
          cssModuleHmrEntries.push({ request, varName, bindingName });
          return `import ${varName} from ${JSON.stringify(request)};`;
        }
        return `import ${JSON.stringify(request)};`;
      })
      .join("\n");

    output = styleImports + "\n" + output;

    // Custom element mode: attach styles array to component for shadow DOM
    if (isCustomElement) {
      const stylesArray = activeStyles.map((s) => `_style_${s.index}`).join(",");
      output = output.replace(
        /^export default _sfc_main;/m,
        `_sfc_main.styles = [${stylesArray}];\nexport default _sfc_main;`,
      );
    }

    // Add CSS module bindings to component (non-custom-element only)
    if (!isCustomElement && cssModuleHmrEntries.length > 0) {
      const cssModuleSetup = cssModuleHmrEntries
        .map(
          (m) =>
            `_sfc_main.__cssModules = _sfc_main.__cssModules || {};\n_sfc_main.__cssModules[${JSON.stringify(m.bindingName)}] = ${m.varName};`,
        )
        .join("\n");

      // CSS Module HMR: accept changes and trigger rerender instead of full reload
      const cssModuleHmr =
        options.hmr && compiled.scopeId
          ? cssModuleHmrEntries
              .map((m) =>
                genCSSModuleHotReloadCode(
                  compiled.scopeId,
                  JSON.stringify(m.request),
                  m.varName,
                  m.bindingName,
                ),
              )
              .join("\n")
          : "";

      output = output.replace(
        /^export default _sfc_main;/m,
        `${cssModuleSetup}\n${cssModuleHmr}\nexport default _sfc_main;`,
      );
    }
  }

  // Expose __file for Vue DevTools (relative path in dev, not in production for security)
  if (options.filePath && !options.isProduction) {
    const relativePath = options.rootContext
      ? path.relative(options.rootContext, options.filePath).replace(/\\/g, "/")
      : path.basename(options.filePath);
    output = output.replace(
      /^export default _sfc_main;/m,
      `_sfc_main.__file = ${JSON.stringify(relativePath)};\nexport default _sfc_main;`,
    );
  }

  // Inject HMR code (must be before export default, after all other setup)
  if (options.hmr && compiled.scopeId) {
    output = output.replace(
      /^export default _sfc_main;/m,
      `${genHotReloadCode(compiled.scopeId)}\nexport default _sfc_main;`,
    );
  }

  // Inject custom block imports
  if (compiled.customBlocks.length > 0) {
    const customBlockImports = compiled.customBlocks
      .map((block, index) => {
        const queryParts = [
          "vue",
          `type=${block.type}`,
          `index=${index}`,
          ...(block.src ? ["src=true"] : []),
        ];
        // Include extra attributes in the query (e.g., lang)
        for (const [key, value] of Object.entries(block.attrs)) {
          if (key === "src") continue; // already handled
          if (value === true) {
            queryParts.push(key);
          } else {
            queryParts.push(`${key}=${value}`);
          }
        }

        const queryStr = queryParts.join("&");
        // Always use the .vue file itself as the request path, even for
        // external-src blocks.  This ensures the import matches the .vue
        // test rule and enters the vize loader, where the custom block
        // handler reads the external file via block.src at loader time.
        const request = options.requestPath;
        return (
          `import block${index} from ${JSON.stringify(`${request}?${queryStr}`)};\n` +
          `if (typeof block${index} === 'function') block${index}(_sfc_main);`
        );
      })
      .join("\n");

    // Insert before "export default _sfc_main;"
    output = output.replace(
      /^export default _sfc_main;/m,
      `${customBlockImports}\nexport default _sfc_main;`,
    );
  }

  return output;
}
