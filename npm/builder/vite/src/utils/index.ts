import { createHash } from "node:crypto";
import type { CompiledModule, StyleBlockInfo } from "../types.ts";
import { type HmrUpdateType, generateHmrCode } from "../hmr.ts";
import {
  analyzeModuleOutput,
  insertBeforeSfcMainDefaultExport,
  rewriteDefaultExportToSfcMain,
} from "./module-output.ts";
import { MappedModule, parseSourceMap, type SourceMapV3 } from "./source-map.ts";

// Re-export CSS utilities for backward compatibility
export { resolveCssImports, type CssAliasRule } from "./css.ts";
export { createFilter } from "./filter.ts";

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

function needsCssPipeline(block: StyleBlockInfo): boolean {
  return block.content.includes("@apply");
}

/**
 * Check if any style blocks in the compiled module require delegation to
 * Vite's CSS pipeline (preprocessor, CSS Modules, or PostCSS transforms).
 */
export function hasDelegatedStyles(compiled: CompiledModule): boolean {
  if (!compiled.styles) return false;
  return compiled.styles.some((s) => needsPreprocessor(s) || isCssModule(s) || needsCssPipeline(s));
}

function supportsTemplateOnlyHmr(output: string): boolean {
  return /(?:^|\n)(?:_sfc_main|__sfc__)\.render\s*=\s*render\b/m.test(output);
}

/**
 * Prepend a runtime `<style>` injection for plain CSS to a module's output.
 *
 * This is the same inline-CSS path plain SFC `<style>` blocks use in
 * {@link generateOutput}: a guarded `document.createElement('style')` keyed by a
 * stable id so the rule is injected once and idempotently. `styleKey` seeds the
 * element id. Used for both SFC plain CSS and JSX `<style scoped>` CSS (#1495,
 * #1533), whose content is already scope-rewritten by the compiler.
 */
export function prependInlineStyleInjection(output: string, css: string, styleKey: string): string {
  const cssCode = JSON.stringify(css);
  const cssId = JSON.stringify(`vize-style-${styleKey}`);
  return `
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

function insertAfterStaticImports(output: string, imports: string): string {
  let insertAt = 0;
  for (const match of output.matchAll(/^import\b[^\n]*(?:\r?\n|$)/gm)) {
    insertAt = (match.index ?? 0) + match[0].length;
  }

  if (insertAt === 0) {
    return `${imports}\n${output}`;
  }

  return `${output.slice(0, insertAt)}${imports}\n${output.slice(insertAt)}`;
}

export function generateScopeId(filename: string): string {
  const hash = createHash("sha256").update(filename).digest("hex");
  return hash.slice(0, 8);
}

export interface GenerateOutputOptions {
  isProduction: boolean;
  isDev: boolean;
  ssr?: boolean;
  hmrUpdateType?: HmrUpdateType;
  extractCss?: boolean;
  /**
   * Absolute path of the source .vue file.
   * Required for generating virtual style imports for preprocessor/CSS Modules delegation.
   */
  filePath?: string;
}

/**
 * Whether the SFC's `<style>` blocks are handed to Vite as virtual imports.
 *
 * Some blocks require Vite's CSS pipeline (preprocessor or CSS Modules), and a
 * production client build routes plain CSS through it too so nesting,
 * minification, and chunk ownership still apply. In both cases the blocks are
 * emitted as imports and `compiled.css` is not used.
 */
function usesStyleImports(compiled: CompiledModule, options: GenerateOutputOptions): boolean {
  return (
    !!options.filePath &&
    !!compiled.styles?.length &&
    (hasDelegatedStyles(compiled) || (!options.ssr && options.isProduction && !!options.extractCss))
  );
}

/**
 * Whether `generateOutput` will embed `compiled.css` in the module.
 *
 * The only consumer of `compiled.css` is the inline `<style>` injection, so this
 * is also the only case in which resolving the CSS's `@import`s affects the
 * output. Callers that would otherwise resolve `compiled.css` eagerly consult
 * this first, which keeps the condition in one place instead of duplicating
 * `generateOutput`'s branch structure at the call site.
 *
 * That guard matters because resolving `@import`s reads and inlines files and
 * crosses the native boundary with the whole stylesheet. A production client
 * build hands plain `<style>` blocks to Vite as virtual imports and an SSR build
 * emits no CSS at all, so in both cases the resolved text was previously built
 * and discarded, once per styled SFC on every build (270 discarded calls on the
 * 300-file bench corpus).
 */
export function embedsInlineCss(compiled: CompiledModule, options: GenerateOutputOptions): boolean {
  return (
    !usesStyleImports(compiled, options) &&
    !options.ssr &&
    !!compiled.css &&
    !(options.isProduction && !!options.extractCss)
  );
}

export function generateOutput(compiled: CompiledModule, options: GenerateOutputOptions): string {
  return generateOutputWithMap(compiled, options).code;
}

/**
 * `generateOutput` plus the source map that describes the module it returns.
 *
 * The compiler's map (`compiled.map`) describes `compiled.code`; every rewrite
 * below goes through {@link MappedModule}, which realigns the map to the lines
 * it inserts, so the returned map describes the returned code (#3399). `map` is
 * `null` when the compiler produced none.
 */
export function generateOutputWithMap(
  compiled: CompiledModule,
  options: GenerateOutputOptions,
): { code: string; map: SourceMapV3 | null } {
  const { isProduction, isDev, ssr, hmrUpdateType, extractCss, filePath } = options;

  const emitted = new MappedModule(compiled.code, parseSourceMap(compiled.map));

  // The native compiler already parsed this module and reports its shape, so
  // the oxc parse here is redundant (#3425). `??` rather than a required field:
  // a `.vpc` cache entry written before this existed reads back without it and
  // simply pays the parse it always paid, so no cache-format bump is needed and
  // the fallback stays exercised.
  const moduleInfo = compiled.moduleShape ?? analyzeModuleOutput(emitted.code);
  const hasExportDefault = moduleInfo.hasDefaultExport;
  const hasNamedRenderExport = moduleInfo.hasNamedRenderExport;
  const hasNamedSsrRenderExport = moduleInfo.hasNamedSsrRenderExport;
  const hasSfcMainDefined = moduleInfo.hasSfcMainDefined;

  if (hasExportDefault && !hasSfcMainDefined) {
    emitted.edit(rewriteDefaultExportToSfcMain(emitted.code, moduleInfo));
    // Add __scopeId for scoped CSS support
    if (compiled.hasScoped && compiled.scopeId) {
      emitted.edit(`${emitted.code}\n_sfc_main.__scopeId = "data-v-${compiled.scopeId}";`);
    }
    emitted.edit(`${emitted.code}\nexport default _sfc_main;`);
  } else if (hasExportDefault && hasSfcMainDefined) {
    // _sfc_main already defined, just add scopeId if needed
    if (compiled.hasScoped && compiled.scopeId) {
      // `emitted.code` is still `compiled.code` on this branch -- nothing above
      // it rewrote the module -- so `moduleInfo`'s offsets describe it exactly and
      // the insertion can reuse them instead of parsing the module again
      // (#3425). The later CSS-modules insertion below cannot: by then the
      // module has been rewritten and the offsets are stale.
      emitted.edit(
        insertBeforeSfcMainDefaultExport(
          emitted.code,
          `_sfc_main.__scopeId = "data-v-${compiled.scopeId}";`,
          { moduleInfo },
        ),
      );
    }
  } else if (!hasExportDefault && !hasSfcMainDefined && hasNamedRenderExport) {
    emitted.edit(`${emitted.code}\nconst _sfc_main = {};`);
    if (compiled.hasScoped && compiled.scopeId) {
      emitted.edit(`${emitted.code}\n_sfc_main.__scopeId = "data-v-${compiled.scopeId}";`);
    }
    emitted.edit(`${emitted.code}\n_sfc_main.render = render;`);
    emitted.edit(`${emitted.code}\nexport default _sfc_main;`);
  } else if (!hasExportDefault && !hasSfcMainDefined && hasNamedSsrRenderExport) {
    emitted.edit(`${emitted.code}\nconst _sfc_main = {};`);
    if (compiled.hasScoped && compiled.scopeId) {
      emitted.edit(`${emitted.code}\n_sfc_main.__scopeId = "data-v-${compiled.scopeId}";`);
    }
    emitted.edit(`${emitted.code}\n_sfc_main.ssrRender = ssrRender;`);
    emitted.edit(`${emitted.code}\nexport default _sfc_main;`);
  }

  // Determine whether to use style imports or inline CSS injection.
  // Production CSS extraction must still import plain CSS through Vite so its
  // CSS pipeline can apply nesting, minification, and chunk graph ownership.
  const useStyleImports = usesStyleImports(compiled, options);

  if (useStyleImports) {
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

    // Keep component/user imports before the SFC's own styles. This matches
    // plugin-vue's cascade order, so parent classes passed to child roots can
    // override child component defaults.
    const allImports = [...styleImports, ...cssModuleImports].join("\n");
    if (allImports) {
      emitted.edit(insertAfterStaticImports(emitted.code, allImports));
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
      emitted.edit(
        insertBeforeSfcMainDefaultExport(emitted.code, cssModuleSetup, {
          normalizeSemicolon: true,
        }),
      );
    }
  } else if (!ssr && compiled.css && !(isProduction && extractCss)) {
    // --- Inline CSS injection (original behavior for plain CSS) ---
    emitted.edit(prependInlineStyleInjection(emitted.code, compiled.css, compiled.scopeId));
  }

  // Add HMR support in development (skip in production)
  if (!isProduction && isDev && hasExportDefault) {
    const effectiveHmrUpdateType =
      hmrUpdateType === "template-only" && !supportsTemplateOnlyHmr(emitted.code)
        ? "full-reload"
        : (hmrUpdateType ?? "full-reload");
    emitted.edit(`${emitted.code}${generateHmrCode(compiled.scopeId, effectiveHmrUpdateType)}`);
  }

  return { code: emitted.code, map: emitted.map };
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
