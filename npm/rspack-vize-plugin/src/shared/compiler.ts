/** Core SFC compilation logic. */

import { createHash } from "node:crypto";
import path from "node:path";
import * as native from "@vizejs/native";
import type { CompiledModule, SfcCompileOptionsNapi } from "../types/index.ts";
import {
  generateScopeId,
  extractStyleBlocks,
  extractCustomBlocks,
  collectTemplateAssetUrls,
} from "./utils.ts";
import { genHotReloadCode, genCSSModuleHotReloadCode } from "./hotReload.ts";

const { compileSfc } = native;

// Compilation Cache

interface CacheEntry {
  contentHash: string;
  result: CompiledModule;
}

/** Content-hash keyed cache for watch mode. */
const compilationCache = new Map<string, CacheEntry>();

function computeContentHash(source: string): string {
  return createHash("sha256").update(source).digest("hex").slice(0, 16);
}

/** Clear the compilation cache. Exposed for testing. */
export function clearCompilationCache(): void {
  compilationCache.clear();
}

/** Compile a .vue file with content-hash caching. */
export function compileFile(
  filePath: string,
  source: string,
  options: {
    sourceMap?: boolean;
    ssr?: boolean;
    vapor?: boolean;
    compilerOptions?: SfcCompileOptionsNapi;
    isCustomElement?: boolean;
    rootContext?: string;
    isProduction?: boolean;
    /** @see VizeLoaderOptions.transformAssetUrls */
    transformAssetUrls?: boolean | Record<string, string[]>;
  } = {},
): CompiledModule {
  // Auto-detect TypeScript
  const autoIsTs = options.compilerOptions?.isTs ?? /<script[^>]*\blang=["']ts["']/.test(source);

  // Composite cache key
  const ssr = options.ssr ?? options.compilerOptions?.ssr ?? false;
  const vapor = options.vapor ?? options.compilerOptions?.vapor ?? false;
  const sourceMap = options.sourceMap ?? options.compilerOptions?.sourceMap ?? true;
  const isCustomElement = options.isCustomElement ?? false;
  const rootCtx = options.rootContext ?? "";
  const isProd = options.isProduction ?? false;
  // Normalize transformAssetUrls for cache key
  const transformAssetUrls = options.transformAssetUrls ?? true;
  const tauKey =
    transformAssetUrls === false
      ? "tau=false"
      : transformAssetUrls === true
        ? "tau=true"
        : `tau=${JSON.stringify(transformAssetUrls)}`;
  const cacheKey = `${filePath}:ssr=${ssr}:vapor=${vapor}:ts=${autoIsTs}:map=${sourceMap}:ce=${isCustomElement}:root=${rootCtx}:prod=${isProd}:${tauKey}`;

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
    vapor,
    isTs: autoIsTs,
    scopeId: hasScoped ? `data-v-${scopeId}` : undefined,
  };

  const result = compileSfc(source, napiOptions);

  const styles = extractStyleBlocks(source);
  const customBlocks = extractCustomBlocks(source);
  const templateAssetUrls = collectTemplateAssetUrls(source, transformAssetUrls);

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
    templateAssetUrls,
    macroArtifacts: result.macroArtifacts ?? [],
  };

  // Only cache successful compilations
  if (compiled.errors.length === 0) {
    compilationCache.set(cacheKey, { contentHash, result: compiled });
  }

  return compiled;
}

/** Generate JS output with style/custom-block imports and optional HMR code. */
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

  // Template static-asset URL rewrite: replace URL string literals in compiled
  // output with import bindings so Rspack can bundle them as assets.
  // Caveat: string-based replacement may also match identical literals in <script>.
  if (compiled.templateAssetUrls.length > 0) {
    for (const { url, varName } of compiled.templateAssetUrls) {
      // Split hash fragment for Rspack module resolution
      const hashIdx = url.indexOf("#");
      const fragment = hashIdx >= 0 ? url.slice(hashIdx) : "";
      const replacement = fragment ? `${varName} + ${JSON.stringify(fragment)}` : varName;

      const escaped = url.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
      output = output.replace(new RegExp(`"${escaped}"`, "g"), replacement);
      output = output.replace(new RegExp(`'${escaped}'`, "g"), replacement);
    }
  }

  // Handle export default transformation
  const exportDefaultRegex = /^export default /m;
  const hasExportDefault = exportDefaultRegex.test(output);
  const hasSfcMainDefined = /\bconst\s+_sfc_main\s*=/.test(output);

  if (hasExportDefault && !hasSfcMainDefined) {
    output = output.replace(exportDefaultRegex, "const _sfc_main = ");
    if (compiled.hasScoped && compiled.scopeId) {
      output += `\n_sfc_main.__scopeId = "data-v-${compiled.scopeId}";`;
    }
    output += "\nexport default _sfc_main;";
  } else if (hasExportDefault && hasSfcMainDefined) {
    if (compiled.hasScoped && compiled.scopeId) {
      output = output.replace(
        /^export default _sfc_main/m,
        `_sfc_main.__scopeId = "data-v-${compiled.scopeId}";\nexport default _sfc_main`,
      );
    }
  }

  // Inject style imports
  if (compiled.styles.length > 0) {
    if (isCustomElement) {
      // Custom element mode: <style module> is not supported
      const hasModule = compiled.styles.some((s) => s.module);
      if (hasModule) {
        throw new Error(`[vize] <style module> is not supported in custom elements mode.`);
      }
    }

    // Only one unnamed <style module> allowed
    const unnamedModuleCount = compiled.styles.filter((s) => s.module === true).length;
    if (unnamedModuleCount > 1) {
      throw new Error(
        `[vize] Found ${unnamedModuleCount} unnamed <style module> blocks. ` +
          `Only one unnamed <style module> is allowed per SFC. ` +
          `Use named modules instead: <style module="name">`,
      );
    }

    const activeStyles = compiled.styles.filter((style) => style.src || /\S/.test(style.content));

    const cssModuleHmrEntries: {
      request: string;
      varName: string;
      bindingName: string;
    }[] = [];

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
          // Use _cssModule_<n> as binding since module name may not be a valid identifier
          const varName = `_cssModule_${style.index}`;
          cssModuleHmrEntries.push({ request, varName, bindingName });
          return `import ${varName} from ${JSON.stringify(request)};`;
        }
        return `import ${JSON.stringify(request)};`;
      })
      .join("\n");

    output = styleImports + "\n" + output;

    // Custom element: attach styles for shadow DOM
    if (isCustomElement) {
      const stylesArray = activeStyles.map((s) => `_style_${s.index}`).join(",");
      output = output.replace(
        /^export default _sfc_main;/m,
        `_sfc_main.styles = [${stylesArray}];\nexport default _sfc_main;`,
      );
    }

    if (!isCustomElement && cssModuleHmrEntries.length > 0) {
      const cssModuleSetup = cssModuleHmrEntries
        .map(
          (m) =>
            `_sfc_main.__cssModules = _sfc_main.__cssModules || {};\n_sfc_main.__cssModules[${JSON.stringify(m.bindingName)}] = ${m.varName};`,
        )
        .join("\n");

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

  // Expose __file for Vue DevTools (dev only)
  if (options.filePath && !options.isProduction) {
    const relativePath = options.rootContext
      ? path.relative(options.rootContext, options.filePath).replace(/\\/g, "/")
      : path.basename(options.filePath);
    output = output.replace(
      /^export default _sfc_main;/m,
      `_sfc_main.__file = ${JSON.stringify(relativePath)};\nexport default _sfc_main;`,
    );
  }

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
        for (const [key, value] of Object.entries(block.attrs)) {
          if (key === "src") continue;
          if (value === true) {
            queryParts.push(key);
          } else {
            queryParts.push(`${key}=${value}`);
          }
        }

        const queryStr = queryParts.join("&");
        const request = options.requestPath;
        return (
          `import block${index} from ${JSON.stringify(`${request}?${queryStr}`)};\n` +
          `if (typeof block${index} === 'function') block${index}(_sfc_main);`
        );
      })
      .join("\n");

    output = output.replace(
      /^export default _sfc_main;/m,
      `${customBlockImports}\nexport default _sfc_main;`,
    );
  }

  // Prepend asset URL import declarations
  if (compiled.templateAssetUrls.length > 0) {
    const assetImports = compiled.templateAssetUrls
      .map(({ url, varName }) => {
        let importPath = url.startsWith("~") ? url.slice(1) : url;
        const hashIdx = importPath.indexOf("#");
        if (hashIdx >= 0) importPath = importPath.slice(0, hashIdx);
        return `import ${varName} from ${JSON.stringify(importPath)};`;
      })
      .join("\n");
    output = assetImports + "\n" + output;
  }

  return output;
}
