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

function supportsTemplateOnlyHmr(output: string): boolean {
  return /(?:^|\n)(?:_sfc_main|__sfc__)\.render\s*=\s*render\b/m.test(output);
}

/**
 * Prepend a runtime `<style>` injection for plain CSS to a module's output.
 *
 * This is the same inline-CSS path plain SFC `<style>` blocks use (see
 * {@link generateOutput}): a guarded `document.createElement("style")` keyed by
 * a stable id, so the rule is appended once and idempotently. `styleKey` seeds
 * the element id (deduping re-injection across HMR/multiple imports). Used for
 * both SFC plain CSS and JSX `<style scoped>` CSS (#1495, #1533), whose content
 * is already scope-rewritten by the compiler.
 */
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
  isProduction: boolean;
  isDev: boolean;
  /**
   * A host that bundles stylesheets gets every block as a style module and
   * writes the CSS; with `"none"`, the compiled CSS is injected at run time.
   */
  hostCss: HostCss;
  filePath: string;
  /** Overrides the `?vue&type=style` import specifier (webpack style files). */
  styleRequest?: (block: StyleBlockInfo, scoped: string | null) => string;
}

export function generateOutput(compiled: CompiledModule, options: GenerateOutputOptions): string {
  const { isProduction, isDev, hostCss, filePath, styleRequest } = options;

  let output = compiled.code;
  const exportDefaultRegex = /^export default /m;
  const hasExportDefault = exportDefaultRegex.test(output);
  const hasNamedRenderExport = /^export function render\b/m.test(output);
  const hasSfcMainDefined = /\bconst\s+_sfc_main\s*=/.test(output);

  if (hasExportDefault && !hasSfcMainDefined) {
    output = output.replace(exportDefaultRegex, "const _sfc_main = ");
    if (compiled.hasScoped) {
      output += `\n_sfc_main.__scopeId = "data-v-${compiled.scopeId}";`;
    }
    output += "\nexport default _sfc_main;";
  } else if (hasExportDefault && hasSfcMainDefined && compiled.hasScoped) {
    output = output.replace(
      /^export default _sfc_main/m,
      `_sfc_main.__scopeId = "data-v-${compiled.scopeId}";\nexport default _sfc_main`,
    );
  } else if (!hasExportDefault && !hasSfcMainDefined && hasNamedRenderExport) {
    output += "\nconst _sfc_main = {};";
    if (compiled.hasScoped) {
      output += `\n_sfc_main.__scopeId = "data-v-${compiled.scopeId}";`;
    }
    output += "\n_sfc_main.render = render;";
    output += "\nexport default _sfc_main;";
  }

  if (hostCss !== "none") {
    const styleImports: string[] = [];
    const cssModuleImports: string[] = [];

    for (const block of compiled.styles) {
      if (block.src) {
        styleImports.push(externalStyleImport(block.src));
        continue;
      }
      const scoped = block.scoped ? `data-v-${compiled.scopeId}` : null;
      const styleFile = styleRequest?.(block, scoped);
      const styleId = styleFile ?? styleRequestId(filePath, block, scoped, hostCss);

      if (isCssModule(block)) {
        const bindingName = typeof block.module === "string" ? block.module : "$style";
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
        .filter((block) => isCssModule(block))
        .map((block) => {
          const bindingName = typeof block.module === "string" ? block.module : "$style";
          return `_sfc_main.__cssModules = _sfc_main.__cssModules || {};\n_sfc_main.__cssModules[${JSON.stringify(bindingName)}] = ${bindingName};`;
        })
        .join("\n");

      output = output.replace(
        /^export default _sfc_main;?$/m,
        `${cssModuleSetup}\nexport default _sfc_main;`,
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

  if (!isProduction && isDev && hasExportDefault && supportsTemplateOnlyHmr(output)) {
    output += "";
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
