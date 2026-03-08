import { createHash } from "node:crypto";
import type { CompiledModule, StyleBlockInfo } from "../types.js";
import { type HmrUpdateType, generateHmrCode } from "../hmr.js";

// Re-export CSS utilities for backward compatibility
export { resolveCssImports, type CssAliasRule } from "./css.js";

/** Known CSS preprocessor languages that must be delegated to Vite */
const PREPROCESSOR_LANGS = new Set(["scss", "sass", "less", "stylus", "styl"]);

/** Check if a style block requires Vite's preprocessor pipeline */
function needsPreprocessor(block: StyleBlockInfo): boolean {
  return block.lang !== null && PREPROCESSOR_LANGS.has(block.lang);
}

/** Check if a style block uses CSS Modules */
function isCssModule(block: StyleBlockInfo): boolean {
  return block.module !== false;
}

/**
 * Check if any style blocks in the compiled module require delegation to
 * Vite's CSS pipeline (preprocessor or CSS Modules).
 */
export function hasDelegatedStyles(compiled: CompiledModule): boolean {
  if (!compiled.styles) return false;
  return compiled.styles.some((s) => needsPreprocessor(s) || isCssModule(s));
}

function supportsTemplateOnlyHmr(output: string): boolean {
  return /(?:^|\n)(?:_sfc_main|__sfc__)\.render\s*=\s*render\b/m.test(output);
}

export function generateScopeId(filename: string): string {
  const hash = createHash("sha256").update(filename).digest("hex");
  return hash.slice(0, 8);
}

export function createFilter(
  include?: string | RegExp | (string | RegExp)[],
  exclude?: string | RegExp | (string | RegExp)[],
): (id: string) => boolean {
  const includePatterns = include ? (Array.isArray(include) ? include : [include]) : [/\.vue$/];
  const excludePatterns = exclude
    ? Array.isArray(exclude)
      ? exclude
      : [exclude]
    : [/node_modules/];

  return (id: string) => {
    const matchInclude = includePatterns.some((pattern) =>
      typeof pattern === "string" ? id.includes(pattern) : pattern.test(id),
    );
    const matchExclude = excludePatterns.some((pattern) =>
      typeof pattern === "string" ? id.includes(pattern) : pattern.test(id),
    );
    return matchInclude && !matchExclude;
  };
}

export interface GenerateOutputOptions {
  isProduction: boolean;
  isDev: boolean;
  hmrUpdateType?: HmrUpdateType;
  extractCss?: boolean;
  /**
   * Absolute path of the source .vue file.
   * Required for generating virtual style imports for preprocessor/CSS Modules delegation.
   */
  filePath?: string;
}

export function generateOutput(compiled: CompiledModule, options: GenerateOutputOptions): string {
  const { isProduction, isDev, hmrUpdateType, extractCss, filePath } = options;

  let output = compiled.code;

  // Rewrite "export default" to named variable for HMR
  // Use regex to match only line-start "export default" (not inside strings)
  const exportDefaultRegex = /^export default /m;
  const hasExportDefault = exportDefaultRegex.test(output);
  const hasNamedRenderExport = /^export function render\b/m.test(output);

  // Check if _sfc_main is already defined (Case 2: non-script-setup SFCs)
  // In this case, the compiler already outputs: const _sfc_main = ...; export default _sfc_main
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
      // Insert scopeId assignment before the export default line
      output = output.replace(
        /^export default _sfc_main/m,
        `_sfc_main.__scopeId = "data-v-${compiled.scopeId}";\nexport default _sfc_main`,
      );
    }
  } else if (!hasExportDefault && !hasSfcMainDefined && hasNamedRenderExport) {
    output += "\nconst _sfc_main = {};";
    if (compiled.hasScoped && compiled.scopeId) {
      output += `\n_sfc_main.__scopeId = "data-v-${compiled.scopeId}";`;
    }
    output += "\n_sfc_main.render = render;";
    output += "\nexport default _sfc_main;";
  }

  // Determine whether to use delegated style imports or inline CSS injection
  const useDelegatedStyles = hasDelegatedStyles(compiled) && filePath;

  if (useDelegatedStyles) {
    // --- Delegated style handling ---
    // Some style blocks require Vite's CSS pipeline (preprocessor or CSS Modules).
    // Emit virtual style imports for ALL blocks so Vite handles them uniformly.
    const styleImports: string[] = [];
    const cssModuleImports: string[] = [];

    for (const block of compiled.styles!) {
      const lang = block.lang ?? "css";
      const params = new URLSearchParams();
      params.set("vue", "");
      params.set("type", "style");
      params.set("index", String(block.index));
      if (block.scoped) params.set("scoped", `data-v-${compiled.scopeId}`);
      params.set("lang", lang);

      if (isCssModule(block)) {
        // CSS Modules: import as a named binding
        const bindingName = typeof block.module === "string" ? block.module : "$style";
        params.set("module", typeof block.module === "string" ? block.module : "");
        const importUrl = `${filePath}?${params.toString()}`;
        cssModuleImports.push(`import ${bindingName} from ${JSON.stringify(importUrl)};`);
      } else {
        // Side-effect import: Vite will inject the CSS
        const importUrl = `${filePath}?${params.toString()}`;
        styleImports.push(`import ${JSON.stringify(importUrl)};`);
      }
    }

    // Prepend style imports
    const allImports = [...styleImports, ...cssModuleImports].join("\n");
    if (allImports) {
      output = allImports + "\n" + output;
    }

    // Inject CSS module bindings into the component
    if (cssModuleImports.length > 0) {
      // Extract binding names from the CSS module imports
      const moduleBindings: { name: string; bindingName: string }[] = [];
      for (const block of compiled.styles!) {
        if (isCssModule(block)) {
          const bindingName = typeof block.module === "string" ? block.module : "$style";
          moduleBindings.push({ name: bindingName, bindingName });
        }
      }

      // Add computed properties to the component for CSS module bindings
      // This makes `$style` available in templates via `useCssModule()`
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
  } else if (compiled.css && !(isProduction && extractCss)) {
    // --- Inline CSS injection (original behavior for plain CSS) ---
    const cssCode = JSON.stringify(compiled.css);
    const cssId = JSON.stringify(`vize-style-${compiled.scopeId}`);
    output = `
export const __vize_css__ = ${cssCode};
const __vize_css_id__ = ${cssId};
(function() {
  if (typeof document !== 'undefined') {
    let style = document.getElementById(__vize_css_id__);
    if (!style) {
      style = document.createElement('style');
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

  // Add HMR support in development (skip in production)
  if (!isProduction && isDev && hasExportDefault) {
    const effectiveHmrUpdateType =
      hmrUpdateType === "template-only" && !supportsTemplateOnlyHmr(output)
        ? "full-reload"
        : (hmrUpdateType ?? "full-reload");
    output += generateHmrCode(compiled.scopeId, effectiveHmrUpdateType);
  }

  return output;
}

/**
 * Legacy generateOutput signature for backward compatibility.
 */
export function generateOutputLegacy(
  compiled: CompiledModule,
  isProduction: boolean,
  isDev: boolean,
): string {
  return generateOutput(compiled, { isProduction, isDev });
}
