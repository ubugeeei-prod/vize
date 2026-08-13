import type { UserConfigExport } from "../../../cli/src/types/index.ts";
import type { ExperimentalPluginOptions } from "./experimental-options.ts";
import type { VizeCompatibilityOptions, VizeVueVersion } from "./compatibility-types.ts";
import type { VizeInspectorOptions } from "./inspector-types.ts";
import type {
  VitePluginVueCustomElementOption,
  VitePluginVueScriptOptions,
  VitePluginVueStyleOptions,
  VitePluginVueTemplateOptions,
} from "./plugin-vue-types.ts";
import type { VizeVueFeatures } from "./vue-features.ts";

export type {
  VizeInspectorLintPlanProvider,
  VizeInspectorLintPlanRequest,
  VizeInspectorOptions,
} from "./inspector-types.ts";

export type {
  VizeConfig,
  ResolvedVizeConfig,
  LoadConfigOptions,
  ConfigEnv,
  UserConfigExport,
} from "../../../cli/src/types/index.ts";
export type { VizeCompatibilityOptions, VizeVueVersion } from "./compatibility-types.ts";
export type {
  VitePluginVueComponentIdGenerator,
  VitePluginVueCustomElementOption,
  VitePluginVueFilterPattern,
  VitePluginVueScriptOptions,
  VitePluginVueStyleOptions,
  VitePluginVueTemplateCompilerOptions,
  VitePluginVueTemplateOptions,
} from "./plugin-vue-types.ts";
export type {
  CompileSfcFn,
  CompiledModule,
  MacroArtifact,
  NativeStyleBlockInfo,
  SfcCompileOptionsNapi,
  SfcCompileResultNapi,
  StyleBlockInfo,
} from "./sfc-types.ts";

export type {
  CompileJsxFn,
  JsxCompileOptionsNapi,
  JsxCompileResultNapi,
  JsxScopedStyleNapi,
} from "./jsx-types.ts";

export interface VizeOptions extends ExperimentalPluginOptions {
  /**
   * Inline shared Vize config for Vite Plus-first projects.
   * Direct plugin options still take precedence over these values.
   */
  config?: UserConfigExport;

  /** Development inspector integrations exposed through Vite's dev server. */
  inspector?: VizeInspectorOptions;

  /** Vue runtime feature flags compatible with `@vitejs/plugin-vue`. */
  features?: VizeVueFeatures;

  /** `@vitejs/plugin-vue` script option bag accepted for drop-in config parity. */
  script?: VitePluginVueScriptOptions;

  /** `@vitejs/plugin-vue` template option bag. */
  template?: VitePluginVueTemplateOptions;

  /** `@vitejs/plugin-vue` style option bag. */
  style?: VitePluginVueStyleOptions;

  /** Lower-level compiler override accepted for plugin-vue config parity. */
  compiler?: unknown;

  /**
   * Top-level custom-element matcher supported by `@vitejs/plugin-vue`.
   * @default /\.ce\.vue$/
   */
  customElement?: VitePluginVueCustomElementOption;

  /**
   * Vue major version for the host project.
   *
   * Legacy Vue projects must keep their existing compiler plugin/loader in
   * charge of SFC compilation. Set this to `0.11`, `1`, `2`, or `"legacy"` to
   * make Vize a non-invasive compatibility plugin that does not intercept
   * `.vue` requests or inject Vue 3 bundler defines.
   *
   * @default 3
   */
  vueVersion?: VizeVueVersion;

  /**
   * Opt-in compatibility features for unsupported host/runtime combinations.
   */
  compatibility?: VizeCompatibilityOptions;

  /**
   * Compilation output mode. Use "function" for CDN/global Vue evaluation.
   * @default "module"
   */
  mode?: "module" | "function";

  /**
   * Module name for runtime imports.
   * @default "vue"
   */
  runtimeModuleName?: string;

  /**
   * Global variable name for function/standalone output.
   * @default "Vue"
   */
  runtimeGlobalName?: string;

  /**
   * Override the public base used for dev-time asset URLs such as /@fs paths.
   * Useful for frameworks like Nuxt that serve Vite from a subpath (e.g. /_nuxt/).
   */
  devUrlBase?: string;

  /**
   * Files to include in compilation
   * @default /\.vue$/
   */
  include?: string | RegExp | (string | RegExp)[];

  /**
   * Files to exclude from compilation
   * @default /node_modules/
   */
  exclude?: string | RegExp | (string | RegExp)[];

  /**
   * Force production mode
   * @default auto-detected from Vite config
   */
  isProduction?: boolean;

  ssr?: boolean;

  /**
   * Enable source map generation.
   * @default development on; production off unless Vite's `build.sourcemap` is set
   */
  sourceMap?: boolean;

  /**
   * Enable Vapor mode compilation
   * @default false
   */
  vapor?: boolean;

  /**
   * Default output mode for `.jsx`/`.tsx` components without a `"use vue:*"`
   * directive. Distinct from `vapor` (which targets `.vue` SFCs): a project can
   * keep SFCs on VDOM while defaulting JSX to Vapor, or vice versa. A
   * per-component `"use vue:vapor"` / `"use vue:vdom"` directive overrides it.
   * @default "vdom"
   */
  jsxMode?: "vdom" | "vapor";

  /**
   * JSX semantics for projects migrating from `@vue/babel-plugin-jsx`.
   * `"native"` keeps Vize's defaults; `"babel"` opts into Babel-compatible
   * behavior. Babel compatibility is only defined for VDOM output.
   * @default "native"
   */
  jsxCompat?: "native" | "babel";

  /**
   * Treat lowercase non-HTML tags as custom renderer elements instead of Vue components.
   * Useful for TresJS and other custom renderers.
   * @default false
   */
  customRenderer?: boolean;

  /**
   * Template syntax compatibility mode.
   * @default "standard"
   */
  templateSyntax?: "standard" | "strict" | "quirks";

  /**
   * Root directory to scan for .vue files
   * @default Vite's root
   */
  root?: string;

  /**
   * Glob patterns to scan for .vue files during pre-compilation
   * Use an empty array to disable startup pre-compilation and compile on demand.
   * @default ['**\/*.vue']
   */
  scanPatterns?: string[];

  /**
   * Maximum number of Vue files to compile in a single native batch during
   * pre-compilation. Lower values reduce peak V8 heap usage in large apps.
   * @default 128
   */
  precompileBatchSize?: number;

  /**
   * Glob patterns to ignore during pre-compilation
   * @default ['node_modules/**', 'dist/**', '.git/**', '.nuxt/**', '.output/**', '.nitro/**', 'coverage/**']
   */
  ignorePatterns?: string[];

  /**
   * Config file search mode
   * - 'root': Search only in the project root directory
   * - 'auto': Search from cwd upward until finding a config file
   * - false: Disable config file loading
   * @default 'root'
   */
  configMode?: "root" | "auto" | false;

  /**
   * Custom config file path (overrides automatic search)
   */
  configFile?: string;

  /**
   * Handle .vue files in node_modules (on-demand compilation).
   * When true, vize will compile .vue files from node_modules that other plugins
   * (like vite-plugin-vue-inspector) may import directly.
   * Set to false if another Vue plugin (e.g. Nuxt) handles node_modules .vue files.
   * @default true
   */
  handleNodeModulesVue?: boolean;

  /**
   * Enable debug logging
   * @default false
   */
  debug?: boolean;
}

export type {
  BatchCompileOptionsNapi,
  BatchCompileResultWithFiles,
  BatchFileInput,
  BatchFileResult,
  CompileSfcBatchWithResultsFn,
} from "./batch-types.ts";
