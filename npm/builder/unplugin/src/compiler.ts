import { createHash } from "node:crypto";
import * as native from "@vizejs/native";
import { prependMappedJsxCode } from "../../shared/source-map.ts";
import { styleRequestId } from "./request.ts";
import type {
  CompiledModule,
  HostCss,
  NormalizedVizeUnpluginOptions,
  CachedCompiledModule,
  JsxCompileResultNapi,
  SfcCompileResultNapi,
  StyleBlockInfo,
} from "./types.ts";
import { generateScopeId, prependInlineStyleInjection, toStyleBlockInfo } from "./style.ts";
import {
  assertSfcStyleSrcImports,
  inlineSfcSrcBlocks,
  type SfcSrcResolutions,
} from "../../shared/sfc-src.ts";

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
  const {
    source: inlined,
    dependencies,
    imports,
    styleSrcs,
  } = inlineSfcSrcBlocks(native.parseSfc, filePath, source, srcResolutions);
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

  assertSfcStyleSrcImports(filePath, result.styles, imports);

  const compiled: CompiledModule = {
    code: result.code,
    css: result.css,
    scopeId,
    hasScoped: result.hasScoped,
    templateHash: result.templateHash,
    styleHash: result.styleHash,
    scriptHash: result.scriptHash,
    macroArtifacts: result.macroArtifacts ?? [],
    // An inlined `<style src>` block keeps the file it came from (#11522).
    styles: result.styles.map((style) => ({
      ...toStyleBlockInfo(style),
      srcFile: styleSrcs.get(style.index) ?? null,
    })),
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

export interface JsxStyleOutput {
  hostCss: HostCss;
  /** Overrides the `?vue&type=style` import specifier (webpack style files). */
  styleRequest?: (block: StyleBlockInfo, scoped: null) => string;
}

export function compileJsxModule(
  filePath: string,
  source: string,
  options: NormalizedVizeUnpluginOptions,
  output: JsxStyleOutput,
): { code: string; map: string | null; warnings: string[]; styles: StyleBlockInfo[] } {
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

  // A `.jsx`/`.tsx` component's `<style scoped>` blocks take the SFC `<style>`
  // path (#1495, #1533, #11520): style modules where the host bundles CSS,
  // run-time injection where nothing does. The compiler already scope-rewrites
  // each block, so a style module holds plain, unscoped-again CSS.
  const scoped = result.scopedStyles ?? [];
  const styles = scoped.map((style, index): StyleBlockInfo => ({
    content: style.css,
    src: null,
    lang: "css",
    scoped: false,
    module: false,
    index,
  }));
  let prefix = "";
  if (styles.length > 0 && output.hostCss === "none") {
    const css = styles.map((style) => style.content).join("\n");
    const styleKey = scoped[0].scopeId.replace(/^data-v-/, "");
    prefix = prependInlineStyleInjection("", css, styleKey);
  } else {
    for (const block of styles) {
      const id =
        output.styleRequest?.(block, null) ?? styleRequestId(filePath, block, null, output.hostCss);
      prefix += `import ${JSON.stringify(id)};\n`;
    }
  }
  const { code, map } = prependMappedJsxCode(result.code, result.map ?? null, prefix);

  return {
    code,
    map,
    warnings: result.warnings,
    styles,
  };
}
