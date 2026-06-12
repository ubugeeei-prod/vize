import type { UserConfigExport } from "../../vize/src/types/index.ts";

export type {
  VizeConfig,
  ResolvedVizeConfig,
  LoadConfigOptions,
  ConfigEnv,
  UserConfigExport,
} from "../../vize/src/types/index.ts";

export interface SfcCompileOptionsNapi {
  filename?: string;
  mode?: "module" | "function";
  sourceMap?: boolean;
  ssr?: boolean;
  vapor?: boolean;
  customRenderer?: boolean;
  templateSyntax?: "standard" | "strict" | "quirks";
  runtimeModuleName?: string;
  runtimeGlobalName?: string;
  vueVersion?: string;
  scopeId?: string;
}

export interface MacroArtifact {
  kind: string;
  name: string;
  source: string;
  content: string;
  moduleCode?: string;
  start: number;
  end: number;
}

export interface SfcCompileResultNapi {
  code: string;
  css?: string;
  errors: string[];
  warnings: string[];
  templateHash?: string;
  styleHash?: string;
  scriptHash?: string;
  hasScoped: boolean;
  styles: NativeStyleBlockInfo[];
  macroArtifacts?: MacroArtifact[];
}

export type CompileSfcFn = (
  source: string,
  options?: SfcCompileOptionsNapi,
) => SfcCompileResultNapi;

/** Options for the native `compileJsx` binding. */
export interface JsxCompileOptionsNapi {
  filename?: string;
  lang?: string;
  /**
   * Default JSX output mode (`"vdom"` | `"vapor"`). Mirrors `compiler.jsxMode`
   * and takes precedence over `vapor`. Per-component `"use vue:*"` directives
   * still override it.
   */
  jsxMode?: "vdom" | "vapor";
  vapor?: boolean;
}

/** A JSX component's extracted `<style scoped>` block (#1495, #1533). */
export interface JsxScopedStyleNapi {
  /** Generated scope id, e.g. `data-v-1a2b3c4d`, already applied to the CSS. */
  scopeId: string;
  /** Scope-rewritten CSS, with the `data-v-<hash>` attribute applied. */
  css: string;
}

/** Result of the native `compileJsx` binding. */
export interface JsxCompileResultNapi {
  code: string;
  errors: string[];
  warnings: string[];
  /**
   * Extracted `<style scoped>` blocks across the module's components, in source
   * order (#1495). Empty when no component had a `<style scoped>`. Each entry's
   * CSS is already scope-rewritten; the plugin emits it through the same path
   * SFC `<style>` blocks use (#1533).
   */
  scopedStyles: JsxScopedStyleNapi[];
}

export type CompileJsxFn = (
  source: string,
  options?: JsxCompileOptionsNapi,
) => JsxCompileResultNapi;

export type VizeVueVersion = 0.11 | 1 | 2 | 3 | "legacy";

export interface VizeCompatibilityOptions {
  /**
   * Host Vue version. Vue 0.11/1/2 opt into host-compiler compatibility.
   */
  vueVersion?: VizeVueVersion;
  /**
   * Keep .vue files on the existing Vue compiler for legacy Vue runtimes.
   * @default true when vueVersion is 0.11, 1, 2, or "legacy"
   */
  hostCompiler?: boolean;
  /**
   * Enable function-body output for CDN/global Vue evaluation.
   */
  scriptSetupInStandalone?: boolean;
  /**
   * Allow Vapor output for Options API SFCs when vapor is enabled.
   */
  optionsApiVapor?: boolean;
  /**
   * Override the host Nuxt major when this option object is shared with Nuxt.
   */
  nuxtVersion?: 2 | 3 | 4;
  /**
   * Override the host Webpack major when this option object is shared with unplugin.
   */
  webpackVersion?: 4 | 5;
}

export interface VizeOptions {
  /**
   * Inline shared Vize config for Vite Plus-first projects.
   * Direct plugin options still take precedence over these values.
   */
  config?: UserConfigExport;

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

  /**
   * Enable SSR mode
   * @default false
   */
  ssr?: boolean;

  /**
   * Enable source map generation
   * @default true in development, false in production
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

export interface StyleBlockInfo {
  /** Raw style content (uncompiled for preprocessor langs) */
  content: string;
  /** External source path from `<style src>`, when present */
  src?: string | null;
  /** Language of the style block (e.g., "css", "scss", "less", "sass", "stylus") */
  lang: string | null;
  /** Whether the style block has the scoped attribute */
  scoped: boolean;
  /** CSS Modules: true for unnamed `module`, or the binding name for `module="name"` */
  module: boolean | string;
  /** Index of this style block in the SFC */
  index: number;
}

export interface NativeStyleBlockInfo {
  /** Raw style content (uncompiled for preprocessor langs) */
  content: string;
  /** External source path from `<style src>`, when present */
  src?: string | null;
  /** Language of the style block (e.g., "css", "scss", "less", "sass", "stylus") */
  lang?: string | null;
  /** Whether the style block has the scoped attribute */
  scoped: boolean;
  /** Whether the style block has the module attribute */
  module: boolean;
  /** CSS Modules binding name for named module attributes */
  moduleName?: string | null;
  /** Index of this style block in the SFC */
  index: number;
}

export interface CompiledModule {
  code: string;
  css?: string;
  scopeId: string;
  hasScoped: boolean;
  templateHash?: string;
  styleHash?: string;
  scriptHash?: string;
  /** Compile-time macro artifacts extracted from the source SFC */
  macroArtifacts?: MacroArtifact[];
  /** Per-block style metadata extracted from the source SFC */
  styles?: StyleBlockInfo[];
  /** Files loaded through SFC `src` imports */
  dependencies?: string[];
}

export interface BatchFileInput {
  path: string;
  source: string;
}

export interface BatchFileResult {
  path: string;
  code: string;
  css?: string;
  scopeId: string;
  hasScoped: boolean;
  errors: string[];
  warnings: string[];
  templateHash?: string;
  styleHash?: string;
  scriptHash?: string;
  /** Compile-time macro artifacts extracted from the source SFC */
  macroArtifacts?: MacroArtifact[];
  /** Per-block style metadata extracted from the source SFC */
  styles?: NativeStyleBlockInfo[];
}

export interface BatchCompileOptionsNapi {
  mode?: "module" | "function";
  ssr?: boolean;
  vapor?: boolean;
  customRenderer?: boolean;
  templateSyntax?: "standard" | "strict" | "quirks";
  runtimeModuleName?: string;
  runtimeGlobalName?: string;
  vueVersion?: string;
  threads?: number;
  /**
   * Include per-block style metadata (incl. `styles[].content`). Default OFF.
   * `code`/`css` are always returned; this opts into the extra CSS-modules /
   * preprocessor metadata the bundler pipeline needs.
   */
  includeStyles?: boolean;
  /** Include parsed custom blocks. Default OFF. */
  includeCustomBlocks?: boolean;
  /** Include compile-time macro artifacts. Default OFF. */
  includeMacroArtifacts?: boolean;
  /** Include template/style/script content hashes (for HMR). Default OFF. */
  includeHashes?: boolean;
}

export interface BatchCompileResultWithFiles {
  results: BatchFileResult[];
  successCount: number;
  failedCount: number;
  timeMs: number;
}

export type CompileSfcBatchWithResultsFn = (
  files: BatchFileInput[],
  options?: BatchCompileOptionsNapi,
) => BatchCompileResultWithFiles;
