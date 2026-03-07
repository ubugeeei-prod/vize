/**
 * @vizejs/rspack-plugin
 *
 * High-performance Rspack plugin for Vue SFC compilation powered by Vize.
 *
 * @example
 * ```js
 * // rspack.config.mjs
 * import { VizePlugin } from '@vizejs/rspack-plugin';
 *
 * export default {
 *   plugins: [new VizePlugin()],
 *   module: {
 *     rules: [
 *       {
 *         test: /\.vue$/,
 *         oneOf: [
 *           // Style rules...
 *           { resourceQuery: /vue&type=style/, use: ['@vizejs/rspack-plugin/style-loader'] },
 *           // Main rule
 *           { use: [{ loader: '@vizejs/rspack-plugin/loader' }] }
 *         ]
 *       }
 *     ]
 *   }
 * }
 * ```
 */

// Plugin
export { VizePlugin } from "./plugin/index.js";
export type { VizeRspackPluginOptions } from "./types/index.js";

// Loaders (for direct import)
export { default as vizeLoader } from "./loader/index.js";
export { default as vizeStyleLoader } from "./loader/style-loader.js";
export type { VizeLoaderOptions, VizeStyleLoaderOptions } from "./types/index.js";

// Shared utilities (optional export for advanced usage)
export {
  generateScopeId,
  extractStyleBlocks,
  extractCustomBlocks,
  extractSrcInfo,
  inlineSrcBlocks,
  addScopeToCssFallback,
  matchesPattern,
  createLogger,
} from "./shared/utils.js";

export { genHotReloadCode } from "./shared/hotReload.js";

export { compileFile, generateOutput, clearCompilationCache } from "./shared/compiler.js";

// Preset API
export { createVizeVueRules } from "./preset/rules.js";

// Types
export type {
  CompiledModule,
  StyleBlockInfo,
  CustomBlockInfo,
  SfcSrcInfo,
  SfcCompileOptionsNapi,
  SfcCompileResultNapi,
  LoaderEntry,
  CreateVizeVueRulesOptions,
  VizeStyleLanguage,
} from "./types/index.js";
