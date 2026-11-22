import { extractSfcStyleBlocks, generateSfcScopeId } from "@vizejs/native";
import { styleRequestId } from "./request.ts";
import { assertInjectableCss } from "./style-dependencies.ts";
import type { CompiledModule, HostCss, StyleBlockInfo, StyleBlockNapi } from "./types.ts";

function isCssModule(block: StyleBlockInfo): boolean {
  return block.module !== false;
}

export function generateScopeId(
  filename: string,
  root: string,
  isProduction: boolean,
  source: string,
): string {
  return generateSfcScopeId(filename, root, isProduction, source);
}

export function extractStyleBlocks(source: string): StyleBlockInfo[] {
  return extractSfcStyleBlocks(source).map(toStyleBlockInfo);
}

/** SFC and JSX styles share a stable element ID across imports and hot updates. */
export function prependInlineStyleInjection(output: string, css: string, styleKey: string): string {
  const cssCode = JSON.stringify(css);
  const cssId = JSON.stringify(`vize-style-${styleKey}`);

  return `
export const __vize_css__ = ${cssCode};
const __vize_css_id__ = ${cssId};
(function() {
  if (typeof document !== "undefined") {
    let style = document.getElementById(__vize_css_id__);
    if (!style) {
      style = document.createElement("style");
      style.id = __vize_css_id__;
      style.textContent = __vize_css__;
      document.head.appendChild(style);
    } else {
      style.textContent = __vize_css__;
    }
  }
})();
${output}`;
}

const CSS_MODULE_LOCALS = "__vize_css_module_locals__";
const CSS_MODULE_LOCALS_HELPER = `const ${CSS_MODULE_LOCALS} = (ns) =>
  ns.default?.locals ??
  (ns.default && typeof ns.default === "object" && !Array.isArray(ns.default)
    ? ns.default
    : Object.fromEntries(Object.entries(ns).filter(([key]) => key !== "default")));`;

/** Only an external `<style src>` is left uninlined; it stays an import of that module. */
function externalStyleImport(src: string): string {
  return `import ${JSON.stringify(src)};`;
}

export interface GenerateOutputOptions {
  /**
   * A host that bundles stylesheets gets every block as a style module and
   * writes the CSS; with `"none"`, the compiled CSS is injected at run time.
   */
  hostCss: HostCss;
  filePath: string;
  /** Overrides the `?vue&type=style` import specifier (webpack style files). */
  styleRequest?: (block: StyleBlockInfo, scoped: string | null) => string;
}

const COMPONENT_EXPORT = "export default _sfc_main";

function componentModule(code: string, scopeId: string | null): string {
  const defaultExport = /^export default /m;
  const hasComponent = /\bconst\s+_sfc_main\s*=/.test(code);
  const setScope = scopeId === null ? "" : `_sfc_main.__scopeId = "${scopeId}";\n`;

  if (defaultExport.test(code)) {
    if (hasComponent) {
      return scopeId === null
        ? code
        : code.replace(/^export default _sfc_main/m, `${setScope}${COMPONENT_EXPORT}`);
    }
    const declaration = code.replace(defaultExport, "const _sfc_main = ");
    return `${declaration}\n${setScope}${COMPONENT_EXPORT};`;
  }
  if (!hasComponent && /^export function render\b/m.test(code)) {
    return `${code}\nconst _sfc_main = {};\n${setScope}_sfc_main.render = render;\n${COMPONENT_EXPORT};`;
  }
  return code;
}

function cssModuleBinding(block: StyleBlockInfo): string {
  return typeof block.module === "string" ? block.module : "$style";
}

export function generateOutput(compiled: CompiledModule, options: GenerateOutputOptions): string {
  const { hostCss, filePath, styleRequest } = options;
  const scopeId = `data-v-${compiled.scopeId}`;
  let output = componentModule(compiled.code, compiled.hasScoped ? scopeId : null);

  if (hostCss !== "none") {
    const styleImports: string[] = [];
    const cssModuleImports: string[] = [];

    for (const block of compiled.styles) {
      if (block.src) {
        styleImports.push(externalStyleImport(block.src));
        continue;
      }
      const scoped = block.scoped ? scopeId : null;
      const styleFile = styleRequest?.(block, scoped);
      const styleId = styleFile ?? styleRequestId(filePath, block, scoped, hostCss);

      if (isCssModule(block)) {
        const bindingName = cssModuleBinding(block);
        if (styleFile) {
          // css-loader exports class names as named exports (v7 default) or
          // on the default export, depending on its options.
          const namespace = `__vize_css_module_${block.index}__`;
          cssModuleImports.push(
            `import * as ${namespace} from ${JSON.stringify(styleFile)};`,
            `const ${bindingName} = ${CSS_MODULE_LOCALS}(${namespace});`,
          );
        } else {
          cssModuleImports.push(`import ${bindingName} from ${JSON.stringify(styleId)};`);
        }
      } else {
        styleImports.push(`import ${JSON.stringify(styleId)};`);
      }
    }

    if (cssModuleImports.some((line) => line.includes(CSS_MODULE_LOCALS))) {
      cssModuleImports.unshift(CSS_MODULE_LOCALS_HELPER);
    }
    const allImports = [...styleImports, ...cssModuleImports].join("\n");
    if (allImports) {
      output = `${allImports}\n${output}`;
    }

    if (cssModuleImports.length > 0) {
      const cssModuleSetup = compiled.styles
        .filter(isCssModule)
        .map((block) => {
          const bindingName = cssModuleBinding(block);
          return `_sfc_main.__cssModules = _sfc_main.__cssModules || {};\n_sfc_main.__cssModules[${JSON.stringify(bindingName)}] = ${bindingName};`;
        })
        .join("\n");

      output = output.replace(
        /^export default _sfc_main;?$/m,
        `${cssModuleSetup}\n${COMPONENT_EXPORT};`,
      );
    }
  } else {
    // The compiled CSS carries CSS Modules' local names, which the compiled
    // `__cssModules` maps.
    assertInjectableCss(compiled.styles, filePath);
    if (compiled.css) {
      output = prependInlineStyleInjection(output, compiled.css, compiled.scopeId);
    }
    const imports = compiled.styles.flatMap((block) =>
      block.src ? [externalStyleImport(block.src)] : [],
    );
    if (imports.length > 0) output = `${imports.join("\n")}\n${output}`;
  }

  return output;
}

export function toStyleBlockInfo(block: StyleBlockNapi): StyleBlockInfo {
  return {
    content: block.content,
    src: block.src ?? null,
    lang: block.lang ?? null,
    scoped: block.scoped,
    module: block.module ? (block.moduleName ?? true) : false,
    index: block.index,
  };
}
