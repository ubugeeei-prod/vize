export type VizeNuxtPattern = string | RegExp;
export type VizeNuxtVueVersion = 0.11 | 1 | 2 | 3 | "legacy";

export interface VizeNuxtCompilerCompatibilityOptions {
  vueVersion?: VizeNuxtVueVersion;
  hostCompiler?: boolean;
  scriptSetupInStandalone?: boolean;
  optionsApiVapor?: boolean;
  nuxtVersion?: 2 | 3 | 4;
  webpackVersion?: 4 | 5;
}

/**
 * Nuxt-facing mirror of the public `@vizejs/vite-plugin` options.
 *
 * Keeping this shape local lets the Nuxt module expose compiler configuration
 * without requiring the sibling package's generated declaration files to exist
 * during monorepo lint runs.
 */
export interface VizeNuxtCompilerOptions {
  /**
   * Vue major version for the host project.
   *
   * Legacy Vue projects keep the host Vue compiler in charge. When set to
   * `0.11`, `1`, `2`, or `"legacy"`, the underlying Vite plugin runs in
   * compatibility mode and does not intercept `.vue` files.
   */
  vueVersion?: VizeNuxtVueVersion;
  /**
   * Opt-in compatibility features shared with `@vizejs/vite-plugin`.
   */
  compatibility?: VizeNuxtCompilerCompatibilityOptions;
  /** Emit function-body output for CDN/global Vue evaluation. */
  mode?: "module" | "function";
  /** Module name for runtime imports. */
  runtimeModuleName?: string;
  /** Global variable name for standalone/function mode. */
  runtimeGlobalName?: string;
  /** Override the public base used for dev-time asset URLs. */
  devUrlBase?: string;
  /** Files to include in compilation. */
  include?: VizeNuxtPattern | VizeNuxtPattern[];
  /** Files to exclude from compilation. */
  exclude?: VizeNuxtPattern | VizeNuxtPattern[];
  /** Force production mode. */
  isProduction?: boolean;
  /** Enable SSR mode. */
  ssr?: boolean;
  /** Enable source map generation. */
  sourceMap?: boolean;
  /** Enable Vapor mode compilation. */
  vapor?: boolean;
  /**
   * Default output mode for `.jsx`/`.tsx` components without a `"use vue:*"`
   * directive (forwarded to the underlying Vize plugin). @default "vdom"
   */
  jsxMode?: "vdom" | "vapor";
  /** Treat lowercase non-HTML tags as custom renderer elements. */
  customRenderer?: boolean;
  /** Template syntax compatibility mode. */
  templateSyntax?: "standard" | "strict" | "quirks";
  /** Root directory to scan for .vue files. */
  root?: string;
  /** Glob patterns to scan for .vue files during pre-compilation. */
  scanPatterns?: string[];
  /** Maximum number of Vue files to compile in a single native batch. */
  precompileBatchSize?: number;
  /** Glob patterns to ignore during pre-compilation. */
  ignorePatterns?: string[];
  /** Config file search mode. */
  configMode?: "root" | "auto" | false;
  /** Custom config file path. */
  configFile?: string;
  /** Handle .vue files in node_modules during on-demand compilation. */
  handleNodeModulesVue?: boolean;
  /** Enable debug logging. */
  debug?: boolean;
}
