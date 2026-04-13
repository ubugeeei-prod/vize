/**
 * Vize - High-performance Vue.js toolchain in Rust
 *
 * This package provides:
 * - CLI binary for compilation, linting, and formatting
 * - Configuration utilities for programmatic use
 */

// Types
export type {
  VizeConfig,
  CompilerConfig,
  VitePluginConfig,
  LinterConfig,
  TypeCheckerConfig,
  FormatterConfig,
  LspConfig,
  MuseaConfig,
  MuseaVrtConfig,
  MuseaA11yConfig,
  MuseaAutogenConfig,
  GlobalTypesConfig,
  GlobalTypeDeclaration,
  LoadConfigOptions,
  ConfigEnv,
  UserConfigExport,
  MaybePromise,
  LintPreset,
  RuleSeverity,
  RuleCategory,
} from "./types/index.js";

// Config utilities
export {
  CONFIG_FILE_NAMES,
  VIZE_CONFIG_JSON_SCHEMA_PATH,
  VIZE_CONFIG_PKL_SCHEMA_PATH,
  defineConfig,
  loadConfig,
  normalizeGlobalTypes,
} from "./config.js";
