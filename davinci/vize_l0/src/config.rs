//! In-memory Vize configuration values and projections.

mod document;
mod model;
mod normalize;
pub use crate::dialect::VueDialect;
pub use document::ConfigDocument;
pub use model::{
    ArrowParens, AttributeSortOrder, ComponentNameInTemplateCasingOptions, ConfigEntryFiles,
    ConfigEntryIgnore, ConfigExperimentalVueFlags, ConfigFeatureFlags, ConfigLintRuleOptions,
    CustomEventNameCasing, CustomEventNameCasingOptions, EndOfLine, FormatterConfig,
    GlobalTypeDeclaration, GlobalTypesConfig, HtmlSelfClosingHtmlOptions, HtmlSelfClosingOptions,
    HtmlSelfClosingStyle, HyphenationStyle, JsxCompat, JsxMode, LanguageServerConfig,
    LanguageServerUnstableFlags, LibConfig, LibRegistryConfig, LintRuleOptions, LintRuleSeverity,
    LinterConfig, LinterConfigEntry, LinterConfigPlan, LinterConfigPlanWithConfigRuleOptions,
    LinterConfigPlanWithRuleOptions, LinterExecutionOptions, LinterFeatureFlags, LspConfig,
    MuseaDesignToken, MuseaPreferDesignTokensOptions, NoMutatingPropsOptions,
    NoRestrictedGlobalsOptions, NoRestrictedMembersOptions, ParseVueVersionError, QuoteProps,
    ResolvedLinterConfig, ResolvedLinterConfigWithConfigRuleOptions, RestrictedGlobal,
    RestrictedMember, SfcElementOrderGroup, SfcElementOrderOptions, TemplateComponentNameCasing,
    TrailingComma, TypeCheckerConfig, VizeConfig, VueVersion,
};
pub use normalize::normalize_public_config_value;
