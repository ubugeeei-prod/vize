import { createHash } from "node:crypto";
import * as native from "@vizejs/native";
import { prependMappedJsxCode } from "../../shared/source-map.ts";
import type {
  CompiledModule,
  NormalizedVizeUnpluginOptions,
  CachedCompiledModule,
  JsxCompileResultNapi,
  SfcCompileResultNapi,
} from "./types.ts";
import { generateScopeId, prependInlineStyleInjection, toStyleBlockInfo } from "./style.ts";
import { assertSupportedStyleDependencies } from "./style-dependencies.ts";
import { inlineSfcSrcBlocks } from "./src-blocks.ts";
import type { SfcSrcResolutions } from "../../shared/sfc-src.ts";

const { compileSfc, compileJsx } = native as {
  compileSfc: (source: string, options?: Record<string, unknown>) => SfcCompileResultNapi;
  compileJsx: (source: string, options?: Record<string, unknown>) => JsxCompileResultNapi;
};

function buildSignature(options: NormalizedVizeUnpluginOptions): string {
  return [
    options.isProduction ? "1" : "0",
    options.ssr ? "1" : "0",
    options.vapor ? "1" : "0",
    options.customRenderer ? "1" : "0",
    JSON.stringify(options.customElements ?? []),
    options.templateSyntax,
    options.experimentalInTagComments ? "1" : "0",
    options.experimentalPatternedTemplate ? "1" : "0",
    options.experimentalSelfComponent ? "1" : "0",
    options.experimentalStrictSlotChildren ? "1" : "0",
    options.experimentalServerScript ? "1" : "0",
    options.sourceMap ? "1" : "0",
    options.mode,
    options.runtimeModuleName,
    options.runtimeGlobalName,
    String(options.vueVersion),
    options.hostCompiler ? "1" : "0",
    options.root,
  ].join(":");
}

function buildSourceHash(source: string): string {
  return createHash("sha256").update(source).digest("hex");
}

export function compileVueModule(
  filePath: string,
  source: string,
  options: NormalizedVizeUnpluginOptions,
  cache: Map<string, CachedCompiledModule>,
  srcResolutions?: SfcSrcResolutions,
): { compiled: CompiledModule; warnings: string[]; dependencies: string[] } {
  // Hash the inlined source so an edited `src` file invalidates the cache.
  const { source: inlined, dependencies } = inlineSfcSrcBlocks(filePath, source, srcResolutions);
  const sourceHash = buildSourceHash(inlined);
  const signature = buildSignature(options);
  const cached = cache.get(filePath);

  if (cached && cached.sourceHash === sourceHash && cached.signature === signature) {
    return { compiled: cached.compiled, warnings: [], dependencies };
  }

  // The authored source keeps scope ids equal to @vizejs/vite-plugin's.
  const scopeId = generateScopeId(filePath, options.root, options.isProduction, source);
  const result = compileSfc(inlined, {
    filename: filePath,
    mode: options.mode,
    sourceMap: options.sourceMap,
    ssr: options.ssr,
    vapor: options.vapor,
    customRenderer: options.customRenderer,
    customElements: options.customElements,
    templateSyntax: options.templateSyntax,
    experimentalInTagComments: options.experimentalInTagComments,
    experimentalPatternedTemplate: options.experimentalPatternedTemplate,
    experimentalSelfComponent: options.experimentalSelfComponent,
    experimentalStrictSlotChildren: options.experimentalStrictSlotChildren,
    experimentalServerScript: options.experimentalServerScript,
    runtimeModuleName: options.runtimeModuleName,
    runtimeGlobalName: options.runtimeGlobalName,
    vueVersion: String(options.vueVersion),
    scopeId: `data-v-${scopeId}`,
    // Production matches @vizejs/vite-plugin and @vitejs/plugin-vue: render
    // inlined into setup, no dev prop runtime types. SSR keeps a separate render.
    isProd: options.isProduction,
    inlineTemplate: options.isProduction && !options.ssr && options.mode !== "function",
  });

  if (result.errors.length > 0) {
    throw new Error(result.errors.join("\n"));
  }

  assertSupportedStyleDependencies(result.styles, filePath);

  const compiled: CompiledModule = {
    code: result.code,
    css: result.css,
    scopeId,
    hasScoped: result.hasScoped,
    templateHash: result.templateHash,
    styleHash: result.styleHash,
    scriptHash: result.scriptHash,
    macroArtifacts: result.macroArtifacts ?? [],
    styles: result.styles.map(toStyleBlockInfo),
  };

  cache.set(filePath, {
    compiled,
    sourceHash,
    signature,
  });

  return {
    compiled,
    warnings: result.warnings,
    dependencies,
  };
}

export function compileJsxModule(
  filePath: string,
  source: string,
  options: NormalizedVizeUnpluginOptions,
): { code: string; map: string | null; warnings: string[] } {
  const result = compileJsx(source, {
    filename: filePath,
    lang: filePath.endsWith(".tsx") ? "tsx" : "jsx",
    jsxMode: options.jsxMode,
    jsxCompat: options.jsxCompat,
    vapor: options.vapor,
    sourceMap: options.sourceMap,
  });

  if (result.errors.length > 0) {
    throw new Error(result.errors.join("\n"));
  }

  // A `.jsx`/`.tsx` component's `<style scoped>` becomes emitted CSS through the
  // same inline-injection path plain SFC `<style>` blocks use (#1495, #1533).
  // The compiler already scope-rewrites each block (the `data-v-<hash>` attr is
  // baked into the selectors and injected into the render output), so the blocks
  // are concatenated and emitted verbatim.
  const css = (result.scopedStyles ?? []).map((style) => style.css).join("\n");
  let code = result.code;
  let map = result.map ?? null;
  if (css) {
    const styleKey = result.scopedStyles[0].scopeId.replace(/^data-v-/, "");
    ({ code, map } = prependMappedJsxCode(
      code,
      map,
      prependInlineStyleInjection("", css, styleKey),
    ));
  }

  return {
    code,
    map,
    warnings: result.warnings,
  };
}
