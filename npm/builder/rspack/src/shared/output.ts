/** JS module output assembly for compiled SFCs. */

import path from "node:path";
import { createHash } from "node:crypto";
import { rewriteSfcTemplateAssetReferences } from "@vizejs/native";
import type { CompiledModule } from "../types/index.ts";
import { genHotReloadCode, genCSSModuleHotReloadCode } from "./hotReload.ts";
import { bindDefaultExport } from "@vizejs/builder-shared/native-shape";
import { hmrImportSnapshot } from "./module-output.ts";
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
  const shape = compiled.moduleShape;
  if (!shape) {
    throw new Error(
      "[vize] The native compiler's output does not parse, so it reported no module shape.",
    );
  }

  // Bindings are appended to the exported `_sfc_main` after the module. They
  // run before any importer reads the export, so the native shape is the only
  // fact needed and the emitted code is never parsed again (#11583).
  const scope = compiled.hasScoped && compiled.scopeId ? `data-v-${compiled.scopeId}` : null;
  let exportsSfcMain = true;
  if (shape.hasDefaultExport && !shape.hasSfcMainDefined) {
    // The native span describes `compiled.code`, so this is the first edit.
    bindDefaultExport(emitted, shape, "_sfc_main");
    if (scope) emitted.append(`\n_sfc_main.__scopeId = "${scope}";`);
    emitted.append("\nexport default _sfc_main;");
  } else if (shape.hasDefaultExport) {
    exportsSfcMain = shape.defaultExportIsSfcMain;
    if (exportsSfcMain && scope) emitted.append(`\n_sfc_main.__scopeId = "${scope}";`);
  } else if (
    !shape.hasSfcMainDefined &&
    (shape.hasNamedRenderExport || shape.hasNamedSsrRenderExport)
  ) {
    // Template-only SFCs compile to a bare render export; wrap it as Vite does.
    const render = shape.hasNamedRenderExport ? "render" : "ssrRender";
    const scopeId = scope ? `, __scopeId: "${scope}"` : "";
    emitted.append(`\nconst _sfc_main = { ${render}${scopeId} };\nexport default _sfc_main;`);
  } else {
    exportsSfcMain = false;
  }
  const attach = (code: string) => {
    if (exportsSfcMain) emitted.append(`\n${code}`);
  };

  if (compiled.templateAssetUrls.length > 0) {
    emitted.edit(rewriteSfcTemplateAssetReferences(emitted.code, compiled.templateAssetUrls));
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
          // A `<style src>` block's module is the src file itself, as in
          // vue-loader, so its relative references resolve from its directory.
          ...(style.src ? ["src"] : []),
        ];
        const request = `${style.src ?? options.requestPath}?${queryParts.join("&")}`;

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
      attach(`_sfc_main.styles = [${stylesArray}];`);
    }

    if (!isCustomElement && cssModuleHmrEntries.length > 0) {
      const cssModuleInterop = options.nativeCss
        ? ""
        : // css-loader's default export is the CSS array with the class map on
          // `.locals`; style-loader and mini-css-extract export the map itself.
          'const __vize_resolve_css_module__ = (value) => value.default?.locals ?? (value.default && typeof value.default === "object" && !Array.isArray(value.default) ? value.default : value);';
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

      attach(`${cssModuleSetup}\n${cssModuleHmr}`);
    }
  }

  if (options.filePath && !options.isProduction) {
    const relativePath = options.rootContext
      ? path.relative(options.rootContext, options.filePath).replace(/\\/g, "/")
      : path.basename(options.filePath);
    attach(`_sfc_main.__file = ${JSON.stringify(relativePath)};`);
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

    attach(customBlockImports);
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

  if (options.hmr && compiled.scopeId && exportsSfcMain) {
    const metadata = compiled.hmr && {
      ...compiled.hmr,
      module: createHash("sha256").update(emitted.code).digest("hex").slice(0, 16),
      styles: JSON.stringify({
        scoped: compiled.hasScoped,
        blocks: compiled.styles.map(({ content, ...style }) => ({
          ...style,
          active: Boolean(style.src || /\S/.test(content)),
        })),
      }),
    };
    attach(genHotReloadCode(compiled.scopeId, metadata, hmrImportSnapshot(emitted.code) ?? "null"));
  }

  return { code: emitted.code, map: emitted.map };
}
