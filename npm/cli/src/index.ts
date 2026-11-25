/**
 * Vize - High-performance Vue.js toolchain in Rust
 *
 * This package provides:
 * - CLI binary for compilation, linting, and formatting
 * - Configuration utilities for programmatic use
 * - The full template tree for JavaScript tools (`parseTemplateAst`)
 */

// Types
export type {
  VizeConfig,
  CompilerConfig,
  VitePluginConfig,
  LinterConfig,
  TypeCheckerConfig,
  FormatterConfig,
  LanguageServerConfig,
  LspConfig,
  MuseaConfig,
  MuseaVrtConfig,
  MuseaA11yConfig,
  MuseaAutogenConfig,
  GlobalTypesConfig,
  GlobalTypeDeclaration,
  LibConfig,
  LoadConfigOptions,
  ConfigEnv,
  VueConfig,
  VueOptions,
  VueOptionsEntry,
  UserConfig,
  UserConfigEntry,
  UserConfigExport,
  MaybePromise,
  LintPreset,
  RuleSeverity,
  RuleCategory,
  VueVersion,
  CompilerCompatibilityConfig,
  VizeConfigEntry,
  LintRuleName,
  LintRulesConfig,
  UserConfigInput,
  ResolvedVizeConfig,
} from "./types/index.js";

// Config utilities
export {
  CONFIG_FILE_NAMES,
  VITE_CONFIG_FILE_NAMES,
  VIZE_CONFIG_JSON_SCHEMA_PATH,
  VIZE_CONFIG_PKL_SCHEMA_PATH,
  defineConfig,
  loadConfig,
  resolveConfigExport,
  resolveViteConfigExport,
  normalizeGlobalTypes,
} from "./config.js";

// Template tree
export { parseTemplateAst } from "./template.js";
export type {
  TemplateAstDiagnostic,
  TemplateAstOptions,
  TemplateAstResult,
  TemplateAttributeNode,
  TemplateChildNode,
  TemplateCommentNode,
  TemplateDirectiveNode,
  TemplateElementNode,
  TemplateInterpolationNode,
  TemplateNode,
  TemplatePosition,
  TemplatePropNode,
  TemplateRootNode,
  TemplateSimpleExpressionNode,
  TemplateSourceLocation,
  TemplateTextNode,
} from "./template.js";
