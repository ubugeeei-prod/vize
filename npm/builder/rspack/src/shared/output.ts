/** JS module output assembly for compiled SFCs. */

import path from "node:path";
import { createHash } from "node:crypto";
import { rewriteSfcTemplateAssetReferences } from "@vizejs/native";
import type { CompiledModule } from "../types/index.ts";
import { genHotReloadCode, genCSSModuleHotReloadCode } from "./hotReload.ts";
import {
  analyzeModuleOutput,
  hmrImportSnapshot,
  insertBeforeSfcMainDefaultExport,
  rewriteDefaultExportToSfcMain,
} from "./module-output.ts";
import { MappedModule, parseSourceMap, type SourceMapV3 } from "./source-map.ts";

export interface GenerateOutputOptions {
  requestPath: string;
  /** Inject HMR boilerplate using `module.hot` (Rspack/webpack CJS API) */
  hmr?: boolean;
  /** Original file path (for __file exposure in dev mode) */
  filePath?: string;
  /** Whether this is a production build */
  isProduction?: boolean;
  /** Project root context (for computing relative __file path) */
  rootContext?: string;
  /** Whether Rspack native CSS is handling CSS module exports */
  nativeCss?: boolean;
}

/** Generate JS output with style/custom-block imports and optional HMR code. */
export function generateOutput(compiled: CompiledModule, options: GenerateOutputOptions): string {
  return generateOutputWithMap(compiled, options).code;
}

/** Generate module code and preserve native mappings through output assembly. */
export function generateOutputWithMap(
  compiled: CompiledModule,
  options: GenerateOutputOptions,
): { code: string; map: SourceMapV3 | null } {
  const emitted = new MappedModule(compiled.code, parseSourceMap(compiled.map));
  const isCustomElement = compiled.isCustomElement;

  if (compiled.templateAssetUrls.length > 0) {
    emitted.edit(rewriteSfcTemplateAssetReferences(emitted.code, compiled.templateAssetUrls));
  }

  const moduleInfo = analyzeModuleOutput(emitted.code);
  const hasExportDefault = moduleInfo.hasDefaultExport;
  const hasSfcMainDefined = moduleInfo.hasSfcMainDefined;

  if (hasExportDefault && !hasSfcMainDefined) {
    rewriteDefaultExportToSfcMain(emitted);
    if (compiled.hasScoped && compiled.scopeId) {
      emitted.append(`\n_sfc_main.__scopeId = "data-v-${compiled.scopeId}";`);
    }
    emitted.append("\nexport default _sfc_main;");
  } else if (hasExportDefault && hasSfcMainDefined && compiled.hasScoped && compiled.scopeId) {
    insertBeforeSfcMainDefaultExport(
      emitted,
      `_sfc_main.__scopeId = "data-v-${compiled.scopeId}";`,
    );
  }

  if (compiled.styles.length > 0) {
    if (isCustomElement && compiled.styles.some((style) => style.module)) {
      throw new Error(`[vize] <style module> is not supported in custom elements mode.`);
    }

    const unnamedModuleCount = compiled.styles.filter((style) => style.module === true).length;
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
        const request = `${options.requestPath}?${queryParts.join("&")}`;

        if (isCustomElement) {
          return `import _style_${style.index} from ${JSON.stringify(request)};`;
        }

        if (style.module) {
          const bindingName = typeof style.module === "string" ? style.module : "$style";
          const varName = `_cssModule_${style.index}`;
          cssModuleHmrEntries.push({ request, varName, bindingName });
          return `import * as ${varName} from ${JSON.stringify(request)};`;
        }
        return `import ${JSON.stringify(request)};`;
      })
      .join("\n");

    emitted.prepend(`${styleImports}\n`);

    if (isCustomElement) {
      const stylesArray = activeStyles.map((style) => `_style_${style.index}`).join(",");
      insertBeforeSfcMainDefaultExport(emitted, `_sfc_main.styles = [${stylesArray}];`, {
        normalizeSemicolon: true,
      });
    }

    if (!isCustomElement && cssModuleHmrEntries.length > 0) {
      const cssModuleInterop = options.nativeCss
        ? ""
        : 'const __vize_resolve_css_module__ = (value) => value.default && typeof value.default === "object" ? value.default : value;';
      const cssModuleSetup = [
        cssModuleInterop,
        ...cssModuleHmrEntries.map(
          (module) =>
            `_sfc_main.__cssModules = _sfc_main.__cssModules || {};\n_sfc_main.__cssModules[${JSON.stringify(module.bindingName)}] = ${options.nativeCss ? module.varName : `__vize_resolve_css_module__(${module.varName})`};`,
        ),
      ]
        .filter(Boolean)
        .join("\n");

      const cssModuleHmr =
        options.hmr && compiled.scopeId
          ? cssModuleHmrEntries
              .map((module) =>
                genCSSModuleHotReloadCode(
                  compiled.scopeId,
                  JSON.stringify(module.request),
                  options.nativeCss
                    ? module.varName
                    : `__vize_resolve_css_module__(${module.varName})`,
                  module.bindingName,
                ),
              )
              .join("\n")
          : "";

      insertBeforeSfcMainDefaultExport(emitted, `${cssModuleSetup}\n${cssModuleHmr}`, {
        normalizeSemicolon: true,
      });
    }
  }

  if (options.filePath && !options.isProduction) {
    const relativePath = options.rootContext
      ? path.relative(options.rootContext, options.filePath).replace(/\\/g, "/")
      : path.basename(options.filePath);
    insertBeforeSfcMainDefaultExport(
      emitted,
      `_sfc_main.__file = ${JSON.stringify(relativePath)};`,
      { normalizeSemicolon: true },
    );
  }

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

        const request = `${options.requestPath}?${queryParts.join("&")}`;
        return (
          `import block${index} from ${JSON.stringify(request)};\n` +
          `if (typeof block${index} === 'function') block${index}(_sfc_main);`
        );
      })
      .join("\n");

    insertBeforeSfcMainDefaultExport(emitted, customBlockImports, {
      normalizeSemicolon: true,
    });
  }

  if (compiled.templateAssetUrls.length > 0) {
    const assetImports = compiled.templateAssetUrls
      .map(({ url, varName }) => {
        let importPath = url.startsWith("~") ? url.slice(1) : url;
        const hashIdx = importPath.indexOf("#");
        if (hashIdx >= 0) importPath = importPath.slice(0, hashIdx);
        return `import ${varName} from ${JSON.stringify(importPath)};`;
      })
      .join("\n");
    emitted.prepend(`${assetImports}\n`);
  }

  if (options.hmr && compiled.scopeId) {
    const metadata = compiled.hmr && {
      ...compiled.hmr,
      module: createHash("sha256").update(emitted.code).digest("hex").slice(0, 16),
    };
    insertBeforeSfcMainDefaultExport(
      emitted,
      genHotReloadCode(compiled.scopeId, metadata, hmrImportSnapshot(emitted.code) ?? "null"),
      { normalizeSemicolon: true },
    );
  }

  return { code: emitted.code, map: emitted.map };
}
