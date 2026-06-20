/** JS module output assembly for compiled SFCs. */

import path from "node:path";
import type { CompiledModule } from "../types/index.ts";
import { genHotReloadCode, genCSSModuleHotReloadCode } from "./hotReload.ts";

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
    /** Whether Rspack native CSS is handling CSS module exports */
    nativeCss?: boolean;
  },
): string {
  let output = compiled.code;
  const isCustomElement = compiled.isCustomElement;

  // Template static-asset URL rewrite: replace URL string literals in compiled
  // output with import bindings so Rspack can bundle them as assets.
  // Caveat: string-based replacement may also match identical literals in <script>.
  if (compiled.templateAssetUrls.length > 0) {
    for (const { url, varName } of compiled.templateAssetUrls) {
      const hashIdx = url.indexOf("#");
      const fragment = hashIdx >= 0 ? url.slice(hashIdx) : "";
      const replacement = fragment ? `${varName} + ${JSON.stringify(fragment)}` : varName;

      const escaped = url.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
      output = output.replace(new RegExp(`"${escaped}"`, "g"), replacement);
      output = output.replace(new RegExp(`'${escaped}'`, "g"), replacement);
    }
  }

  const exportDefaultRegex = /^export default /m;
  const hasExportDefault = exportDefaultRegex.test(output);
  const hasSfcMainDefined = /\bconst\s+_sfc_main\s*=/.test(output);

  if (hasExportDefault && !hasSfcMainDefined) {
    output = output.replace(exportDefaultRegex, "const _sfc_main = ");
    if (compiled.hasScoped && compiled.scopeId) {
      output += `\n_sfc_main.__scopeId = "data-v-${compiled.scopeId}";`;
    }
    output += "\nexport default _sfc_main;";
  } else if (hasExportDefault && hasSfcMainDefined && compiled.hasScoped && compiled.scopeId) {
    output = output.replace(
      /^export default _sfc_main/m,
      `_sfc_main.__scopeId = "data-v-${compiled.scopeId}";\nexport default _sfc_main`,
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
          return options.nativeCss
            ? `import * as ${varName} from ${JSON.stringify(request)};`
            : `import ${varName} from ${JSON.stringify(request)};`;
        }
        return `import ${JSON.stringify(request)};`;
      })
      .join("\n");

    output = styleImports + "\n" + output;

    if (isCustomElement) {
      const stylesArray = activeStyles.map((style) => `_style_${style.index}`).join(",");
      output = output.replace(
        /^export default _sfc_main;/m,
        `_sfc_main.styles = [${stylesArray}];\nexport default _sfc_main;`,
      );
    }

    if (!isCustomElement && cssModuleHmrEntries.length > 0) {
      const cssModuleSetup = cssModuleHmrEntries
        .map(
          (module) =>
            `_sfc_main.__cssModules = _sfc_main.__cssModules || {};\n_sfc_main.__cssModules[${JSON.stringify(module.bindingName)}] = ${module.varName};`,
        )
        .join("\n");

      const cssModuleHmr =
        options.hmr && compiled.scopeId
          ? cssModuleHmrEntries
              .map((module) =>
                genCSSModuleHotReloadCode(
                  compiled.scopeId,
                  JSON.stringify(module.request),
                  module.varName,
                  module.bindingName,
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

    output = output.replace(
      /^export default _sfc_main;/m,
      `${customBlockImports}\nexport default _sfc_main;`,
    );
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
    output = assetImports + "\n" + output;
  }

  return output;
}
