import {
  a as LoaderEntry,
  c as SfcCompileResultNapi,
  d as VizeLoaderOptions,
  f as VizeRspackPluginOptions,
  i as JsxCompileResultNapi,
  l as SfcSrcInfo,
  n as CustomBlockInfo,
  o as MacroArtifact,
  p as VizeStyleLoaderOptions,
  r as JsxCompileOptionsNapi,
  s as SfcCompileOptionsNapi,
  t as CompiledModule,
  u as StyleBlockInfo,
} from "./index-sdINSvBH.mjs";
import vizeLoader from "./loader/index.mjs";
import vizeJsxLoader from "./loader/jsx-loader.mjs";
import vizeStyleLoader from "./loader/style-loader.mjs";
import vizeScopeLoader from "./loader/scope-loader.mjs";
import { Compiler, RuleSetRule } from "@rspack/core";

//#region src/plugin/index.d.ts
declare class VizePlugin {
  static readonly name = "VizePlugin";
  private options;
  constructor(options?: VizeRspackPluginOptions);
  apply(compiler: Compiler): void;
  private shouldHandleFile;
}
//#endregion
//#region src/plugin/ruleCloning.d.ts
interface RuleCloningResult {
  /** Whether rule cloning was performed */
  applied: boolean;
  /** Number of CSS rules cloned */
  clonedCount: number;
  /** Warnings to emit through the infrastructure logger */
  warnings: string[];
}
/**
 * Mutates `rules` in-place: wraps .vue rule into `oneOf`, clones CSS rules for style sub-requests.
 */
declare function applyRuleCloning(
  rules: (RuleSetRule | "...")[],
  nativeCss: boolean,
): RuleCloningResult;
//#endregion
//#region src/shared/utils.d.ts
/** Generate scope ID (8-char SHA256 prefix). Uses relative path for cross-env consistency. */
declare function generateScopeId(
  filename: string,
  rootContext?: string,
  isProduction?: boolean,
  source?: string,
): string;
/** Extract style block metadata from SFC source. */
declare function extractStyleBlocks(source: string): StyleBlockInfo[];
/** Back-compat export for scoped CSS transformation. Delegates to the native CSS pipeline. */
declare function addScopeToCssFallback(css: string, scopeId: string): string;
/** Extract custom block metadata from SFC source (non-script/template/style tags). */
declare function extractCustomBlocks(source: string): CustomBlockInfo[];
/** Extract <script src> and <template src> references from SFC source. */
declare function extractSrcInfo(source: string): SfcSrcInfo;
/** Replace <script src> or <template src> with inline content from external files. */
declare function inlineSrcBlocks(
  source: string,
  scriptContent: string | null,
  templateContent: string | null,
): string;
/** Match a file path against include/exclude patterns. Normalizes backslashes. */
declare function matchesPattern(
  file: string,
  pattern: string | RegExp | (string | RegExp)[] | undefined,
  defaultValue: boolean,
): boolean;
//#endregion
//#region src/shared/hotReload.d.ts
/** HMR code generation for Vue SFCs using `module.hot` (Rspack/webpack CJS API). */
/** Generate `module.hot` HMR boilerplate for a Vue SFC. */
declare function genHotReloadCode(id: string): string;
//#endregion
//#region src/shared/output.d.ts
/** Generate JS output with style/custom-block imports and optional HMR code. */
declare function generateOutput(
  compiled: CompiledModule,
  options: {
    requestPath: string /** Inject HMR boilerplate using `module.hot` (Rspack/webpack CJS API) */;
    hmr?: boolean /** Original file path (for __file exposure in dev mode) */;
    filePath?: string /** Whether this is a production build */;
    isProduction?: boolean /** Project root context (for computing relative __file path) */;
    rootContext?: string /** Whether Rspack native CSS is handling CSS module exports */;
    nativeCss?: boolean;
  },
): string;
//#endregion
//#region src/shared/compiler.d.ts
/** `.jsx`/`.tsx` Vue components routed to the native JSX compiler. */
declare function isJsxFile(filePath: string): boolean;
/**
 * Compile a `.jsx`/`.tsx` Vue module to render code via the native JSX
 * compiler. Mirrors {@link compileFile} but for the JSX lowering path: no custom
 * blocks or asset-url rewriting apply. A component's `<style scoped>` CSS is
 * surfaced (already scope-rewritten) and emitted through the same inline-style
 * injection path the integrations use for plain SFC CSS (#1495, #1533).
 */
declare function compileJsxModule(
  filePath: string,
  source: string,
  options?: {
    jsxMode?: "vdom" | "vapor";
    vapor?: boolean;
    sourceMap?: boolean;
  },
): {
  code: string;
  map: string | null;
  warnings: string[];
};
/** Clear the compilation cache. Exposed for testing. */
declare function clearCompilationCache(): void;
/** Compile a .vue file with content-hash caching. */
declare function compileFile(
  filePath: string,
  source: string,
  options?: {
    sourceMap?: boolean;
    ssr?: boolean;
    vapor?: boolean;
    compilerOptions?: SfcCompileOptionsNapi;
    isCustomElement?: boolean;
    rootContext?: string;
    isProduction?: boolean /** @see VizeLoaderOptions.transformAssetUrls */;
    transformAssetUrls?: boolean | Record<string, string[]>;
  },
): CompiledModule;
//#endregion
export {
  type CompiledModule,
  type CustomBlockInfo,
  type JsxCompileOptionsNapi,
  type JsxCompileResultNapi,
  type LoaderEntry,
  type MacroArtifact,
  type RuleCloningResult,
  type SfcCompileOptionsNapi,
  type SfcCompileResultNapi,
  type SfcSrcInfo,
  type StyleBlockInfo,
  type VizeLoaderOptions,
  VizePlugin,
  type VizeRspackPluginOptions,
  type VizeStyleLoaderOptions,
  addScopeToCssFallback,
  applyRuleCloning,
  clearCompilationCache,
  compileFile,
  compileJsxModule,
  extractCustomBlocks,
  extractSrcInfo,
  extractStyleBlocks,
  genHotReloadCode,
  generateOutput,
  generateScopeId,
  inlineSrcBlocks,
  isJsxFile,
  matchesPattern,
  vizeJsxLoader,
  vizeLoader,
  vizeScopeLoader,
  vizeStyleLoader,
};
