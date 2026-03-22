import type { CompilerConfig } from "./compiler.js";
import type { VitePluginConfig } from "./compiler.js";
import type { LinterConfig, TypeCheckerConfig, FormatterConfig, LspConfig } from "./tools.js";
import type { MuseaConfig } from "./musea.js";
import type { GlobalTypesConfig } from "./loader.js";

// ============================================================================
// Dynamic config support
// ============================================================================

export type MaybePromise<T> = T | Promise<T>;

export interface ConfigEnv {
  mode: string;
  command: "serve" | "build" | "check" | "lint" | "fmt";
  isSsrBuild?: boolean;
}

export type UserConfigExport = VizeConfig | ((env: ConfigEnv) => MaybePromise<VizeConfig>);

// ============================================================================
// Rule severity
// ============================================================================

export type RuleSeverity = "off" | "warn" | "error";

export type RuleCategory = "correctness" | "suspicious" | "style" | "perf" | "a11y" | "security";

export type LintPreset = "happy-path" | "opinionated" | "essential" | "nuxt";

// ============================================================================
// VizeConfig
// ============================================================================

/**
 * Vize configuration options
 */
export interface VizeConfig {
  /**
   * Vue compiler options
   */
  compiler?: CompilerConfig;

  /**
   * Vite plugin options
   */
  vite?: VitePluginConfig;

  /**
   * Linter options
   */
  linter?: LinterConfig;

  /**
   * Type checker options
   */
  typeChecker?: TypeCheckerConfig;

  /**
   * Formatter options
   */
  formatter?: FormatterConfig;

  /**
   * LSP options
   */
  lsp?: LspConfig;

  /**
   * Musea component gallery options
   */
  musea?: MuseaConfig;

  /**
   * Global type declarations
   */
  globalTypes?: GlobalTypesConfig;
}
