export type {
  LintPreset,
  RuleSeverity,
  RuleCategory,
  VueVersion,
  VizeConfig,
  VizeConfigEntry,
  CompilerConfig,
  CompilerCompatibilityConfig,
  VitePluginConfig,
  LinterConfig,
  TypeCheckerConfig,
  FormatterConfig,
  LanguageServerConfig,
  MuseaVrtConfig,
  MuseaA11yConfig,
  MuseaAutogenConfig,
  MuseaConfig,
  MuseaViewport,
  GlobalTypeDeclaration,
  GlobalTypesConfig,
} from "./generated.js";

export type { LintRuleName, LintRulesConfig } from "./rules.js";

/**
 * @deprecated Use `LanguageServerConfig`.
 */
export type LspConfig = import("./generated.js").LanguageServerConfig;

export type {
  MaybePromise,
  ConfigEnv,
  UserConfigInput,
  UserConfigExport,
  ResolvedVizeConfig,
  LoadConfigOptions,
} from "./runtime.js";
