// ============================================================================
// CompilerConfig
// ============================================================================

/**
 * Compiler configuration
 */
export interface CompilerConfig {
  /**
   * Compilation mode
   * @default 'module'
   */
  mode?: "module" | "function";

  /**
   * Enable Vapor mode compilation
   * @default false
   */
  vapor?: boolean;

  /**
   * Treat lowercase non-HTML tags as custom renderer elements instead of Vue components.
   * Useful for TresJS and other custom renderers.
   * @default false
   */
  customRenderer?: boolean;

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
   * Prefix template identifiers with _ctx
   * @default false
   */
  prefixIdentifiers?: boolean;

  /**
   * Hoist static nodes
   * @default true
   */
  hoistStatic?: boolean;

  /**
   * Cache v-on handlers
   * @default true
   */
  cacheHandlers?: boolean;

  /**
   * Enable TypeScript parsing in <script> blocks
   * @default true
   */
  isTs?: boolean;

  /**
   * Script file extension for generated output
   * @default 'ts'
   */
  scriptExt?: "ts" | "js";

  /**
   * Module name for runtime imports
   * @default 'vue'
   */
  runtimeModuleName?: string;

  /**
   * Global variable name for runtime (IIFE builds)
   * @default 'Vue'
   */
  runtimeGlobalName?: string;
}

// ============================================================================
// VitePluginConfig
// ============================================================================

/**
 * Vite plugin configuration
 */
export interface VitePluginConfig {
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
   * Glob patterns to scan for .vue files during pre-compilation
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
}
