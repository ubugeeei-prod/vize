/** @vizejs/rspack-plugin — Rspack plugin for Vue SFC compilation powered by Vize. */

// Plugin
export { VizePlugin } from "./plugin/index.ts";
export { applyRuleCloning } from "./plugin/ruleCloning.ts";
export type { RuleCloningResult } from "./plugin/ruleCloning.ts";
export type { VizeRspackPluginOptions } from "./types/index.ts";

// Loaders (for direct import)
export { default as vizeLoader } from "./loader/index.ts";
export { default as vizeStyleLoader } from "./loader/style-loader.ts";
export { default as vizeScopeLoader } from "./loader/scope-loader.ts";
export type { VizeLoaderOptions, VizeStyleLoaderOptions } from "./types/index.ts";

// Shared utilities (optional export for advanced usage)
export {
  generateScopeId,
  extractStyleBlocks,
  extractCustomBlocks,
  extractSrcInfo,
  inlineSrcBlocks,
  addScopeToCssFallback,
  matchesPattern,
} from "./shared/utils.ts";

export { genHotReloadCode } from "./shared/hotReload.ts";

export { compileFile, generateOutput, clearCompilationCache } from "./shared/compiler.ts";

// Types
export type {
  CompiledModule,
  MacroArtifact,
  StyleBlockInfo,
  CustomBlockInfo,
  SfcSrcInfo,
  SfcCompileOptionsNapi,
  SfcCompileResultNapi,
  LoaderEntry,
} from "./types/index.ts";
