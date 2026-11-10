//! In-memory Vize configuration values and projections.

mod document;
mod model;
mod normalize;
pub use crate::dialect::VueDialect;
pub use document::ConfigDocument;
pub use model::{
    AnchorHasContentOptions, ArrowParens, AttributeHyphenationConfig, AttributeHyphenationOptions,
    AttributeSortOrder, CompilerConfigEntry, ComponentNameInTemplateCasingOptions,
    ConfigEntryFiles, ConfigEntryIgnore, ConfigExperimentalVueFlags, ConfigFeatureFlags,
    ConfigLintRuleOptions, CustomEventNameCasing, CustomEventNameCasingOptions,
    DefineMacrosOrderOptions, DefinePropsDestructuringOptions, EndOfLine,
    EnforceStyleAttributeOptions, FormatterConfig, FormatterPreset, FormatterProfile,
    GlobalTypeDeclaration, GlobalTypesConfig, HtmlButtonHasTypeOptions, HtmlQuoteStyle,
    HtmlQuotesConfig, HtmlQuotesOptions, HtmlSelfClosingHtmlOptions, HtmlSelfClosingOptions,
    HtmlSelfClosingStyle, HyphenationStyle, ImportSortCustomGroup, ImportSortGroup, JsxCompat,
    JsxMode, LabelHasForOptions, LanguageServerConfig, LanguageServerUnstableFlags, LibConfig,
    LibRegistryConfig, LintRuleOptions, LintRuleSeverity, LinterConfig, LinterConfigEntry,
    LinterConfigPlan, LinterConfigPlanWithConfigRuleOptions, LinterConfigPlanWithRuleOptions,
    LinterExecutionOptions, LinterFeatureFlags, LspConfig, MuseaDesignToken,
    MuseaPreferDesignTokensOptions, MustacheInterpolationSpacingStyle,
    NoBareStringsInTemplateOptions, NoMutatingPropsOptions, NoRestrictedGlobalsOptions,
    NoRestrictedMembersOptions, NoRestrictedSyntaxOptions, NoUseVIfWithVForOptions,
    ParseVueVersionError, PropNameCasingOptions, PropNameCasingStyle, PropsDestructureMode,
    QuoteProps, QuotePropsStyle, ResolvedLinterConfig, ResolvedLinterConfigWithConfigRuleOptions,
    RestrictedGlobal, RestrictedMember, SfcElementOrderGroup, SfcElementOrderOptions,
    SlotNameCasingStyle, SortImportsConfig, SortImportsSetting, SortKeysOptions, SortKeysOrder,
    StrictBooleanExpressionsOptions, StyleAttributeMode, TemplateComponentNameCasing,
    TrailingComma, TypeCheckerConfig, VBindStyle, VOnEventHyphenationConfig,
    VOnEventHyphenationOptions, VOnStyle, VSlotNamedStyle, VSlotStyle, VSlotStyleOptions,
    VSlotStyleValue, VSlotUniformStyle, VizeConfig, VueVersion, WarnCustomDirectiveOptions,
};
pub use normalize::normalize_public_config_value;
