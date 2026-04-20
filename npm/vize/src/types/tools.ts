import type { LintPreset, RuleSeverity, RuleCategory } from "./core.js";
import type { LintRulesConfig } from "./rules.js";

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
  rules?: LintRulesConfig;

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
   * @default false
   */
  enabled?: boolean;

  /**
   * Enable linter diagnostics.
   * Prefer this over `diagnostics` for new configs.
   * @default false
   */
  lint?: boolean;

  /**
   * Enable linter diagnostics.
   * @deprecated Use `lint` instead.
   * @default false
   */
  diagnostics?: boolean;

  /**
   * Enable type checking diagnostics and type-aware LSP features.
   * @default false
   */
  typecheck?: boolean;

  /**
   * Enable the editor assistance bundle: completion, hover, navigation,
   * symbols, rename, code lens, semantic tokens, links, folding, inlay hints,
   * and file rename handling. Formatting stays separately opt-in.
   * @default false
   */
  editor?: boolean;

  /**
   * Enable completions.
   * @default false
   */
  completion?: boolean;

  /**
   * Enable hover information
   * @default false
   */
  hover?: boolean;

  /**
   * Enable go-to-definition
   * @default false
   */
  definition?: boolean;

  /**
   * Enable find references
   * @default false
   */
  references?: boolean;

  /**
   * Enable document symbols
   * @default false
   */
  documentSymbols?: boolean;

  /**
   * Enable workspace symbols
   * @default false
   */
  workspaceSymbols?: boolean;

  /**
   * Enable formatting via LSP
   * @default false
   */
  formatting?: boolean;

  /**
   * Enable code actions
   * @default false
   */
  codeActions?: boolean;

  /**
   * Enable rename
   * @default false
   */
  rename?: boolean;

  /**
   * Enable code lens
   * @default false
   */
  codeLens?: boolean;

  /**
   * Enable semantic tokens
   * @default false
   */
  semanticTokens?: boolean;

  /**
   * Enable document links
   * @default false
   */
  documentLinks?: boolean;

  /**
   * Enable folding ranges
   * @default false
   */
  foldingRanges?: boolean;

  /**
   * Enable inlay hints
   * @default false
   */
  inlayHints?: boolean;

  /**
   * Enable file rename edits
   * @default false
   */
  fileRename?: boolean;

  /**
   * Use Corsa for type checking in LSP
   * @default false
   */
  corsa?: boolean;
}
