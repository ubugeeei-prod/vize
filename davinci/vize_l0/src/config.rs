//! In-memory Vize configuration values and projections.

mod document;
mod model;
mod normalize;
pub use crate::dialect::VueDialect;
pub use document::ConfigDocument;
pub use model::{
    ArrowParens, AttributeSortOrder, CompilerConfigEntry, ComponentNameInTemplateCasingOptions,
    ConfigEntryFiles, ConfigEntryIgnore, ConfigExperimentalVueFlags, ConfigFeatureFlags,
    ConfigLintRuleOptions, CustomEventNameCasing, CustomEventNameCasingOptions,
    DefineMacrosOrderOptions, DefinePropsDestructuringOptions, EndOfLine, FormatterConfig,
    FormatterPreset, FormatterProfile, GlobalTypeDeclaration, GlobalTypesConfig, HtmlQuoteStyle,
    HtmlQuotesConfig, HtmlQuotesOptions, HtmlSelfClosingHtmlOptions, HtmlSelfClosingOptions,
    HtmlSelfClosingStyle, HyphenationStyle, ImportSortCustomGroup, ImportSortGroup, JsxCompat,
    JsxMode, LanguageServerConfig, LanguageServerUnstableFlags, LibConfig, LibRegistryConfig,
    LintRuleOptions, LintRuleSeverity, LinterConfig, LinterConfigEntry, LinterConfigPlan,
    LinterConfigPlanWithConfigRuleOptions, LinterConfigPlanWithRuleOptions, LinterExecutionOptions,
    LinterFeatureFlags, LspConfig, MuseaDesignToken, MuseaPreferDesignTokensOptions,
    NoMutatingPropsOptions, NoRestrictedGlobalsOptions, NoRestrictedMembersOptions,
    ParseVueVersionError, PropsDestructureMode, QuoteProps, ResolvedLinterConfig,
    ResolvedLinterConfigWithConfigRuleOptions, RestrictedGlobal, RestrictedMember,
    SfcElementOrderGroup, SfcElementOrderOptions, SortImportsConfig, SortImportsSetting,
    StrictBooleanExpressionsOptions, TemplateComponentNameCasing, TrailingComma, TypeCheckerConfig,
    VBindStyle, VOnStyle, VSlotNamedStyle, VSlotStyle, VSlotStyleOptions, VSlotStyleValue,
    VSlotUniformStyle, VizeConfig, VueVersion,
};
pub use normalize::normalize_public_config_value;
