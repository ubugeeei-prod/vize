import type { LintPreset, RuleSeverity, RuleCategory } from "./core.js";

// ============================================================================
// LinterConfig
// ============================================================================

/**
 * Linter configuration
 */
export interface LinterConfig {
  /**
   * Enable linting
   */
  enabled?: boolean;

  /**
   * Built-in lint preset
   * @default 'happy-path'
   */
  preset?: LintPreset;

  /**
   * Rules to enable/disable
   */
  rules?: Record<string, RuleSeverity>;

  /**
   * Category-level severity overrides
   */
  categories?: Partial<Record<RuleCategory, RuleSeverity>>;
}

// ============================================================================
// TypeCheckerConfig
// ============================================================================

/**
 * Type checker configuration
 */
export interface TypeCheckerConfig {
  /**
   * Enable type checking
   * @default false
   */
  enabled?: boolean;

  /**
   * Enable strict mode
   * @default false
   */
  strict?: boolean;

  /**
   * Check component props
   * @default true
   */
  checkProps?: boolean;

  /**
   * Check component emits
   * @default true
   */
  checkEmits?: boolean;

  /**
   * Check template bindings
   * @default true
   */
  checkTemplateBindings?: boolean;

  /**
   * Path to tsconfig.json
   * @default auto-detected
   */
  tsconfig?: string;

  /**
   * Path to the Corsa binary
   */
  corsaPath?: string;
}

// ============================================================================
// FormatterConfig
// ============================================================================

/**
 * Formatter configuration
 */
export interface FormatterConfig {
  /**
   * Max line width
   * @default 80
   */
  printWidth?: number;

  /**
   * Indentation width
   * @default 2
   */
  tabWidth?: number;

  /**
   * Use tabs for indentation
   * @default false
   */
  useTabs?: boolean;

  /**
   * Print semicolons
   * @default true
   */
  semi?: boolean;

  /**
   * Use single quotes
   * @default false
   */
  singleQuote?: boolean;

  /**
   * Trailing commas
   * @default 'all'
   */
  trailingComma?: "all" | "none" | "es5";
}

// ============================================================================
// LspConfig
// ============================================================================

/**
 * LSP configuration
 */
export interface LspConfig {
  /**
   * Enable LSP
   * @default true
   */
  enabled?: boolean;

  /**
   * Enable diagnostics
   * @default true
   */
  diagnostics?: boolean;

  /**
   * Enable completions
   * @default true
   */
  completion?: boolean;

  /**
   * Enable hover information
   * @default true
   */
  hover?: boolean;

  /**
   * Enable go-to-definition
   * @default true
   */
  definition?: boolean;

  /**
   * Enable formatting via LSP
   * @default true
   */
  formatting?: boolean;

  /**
   * Enable code actions
   * @default true
   */
  codeActions?: boolean;

  /**
   * Use Corsa for type checking in LSP
   * @default false
   */
  corsa?: boolean;
}
