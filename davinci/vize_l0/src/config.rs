//! In-memory Vize configuration values and projections.

mod document;
mod model;
mod normalize;
pub use crate::dialect::VueDialect;
pub use document::ConfigDocument;
pub use model::{
    AnchorHasContentOptions, ArrowParens, AttributeHyphenationConfig, AttributeHyphenationOptions,
    AttributeSortOrder, BlockLangBlock, BlockLangNames, BlockLangOptions, CallAfterAwaitOptions,
    CallAfterAwaitSpec, CompilerConfigEntry, ComponentNameInTemplateCasingOptions,
    ConfigEntryFiles, ConfigEntryIgnore, ConfigExperimentalVueFlags, ConfigFeatureFlags,
    ConfigLintRuleOptions, ConfiguredVueRuleOptions, ConstantLoopConditionMode,
    ConstantLoopConditions, CustomEventNameCasing, CustomEventNameCasingOptions,
    DefineMacrosOrderOptions, DefinePropsDestructuringOptions, EndOfLine,
    EnforceStyleAttributeOptions, FormatterConfig, FormatterPreset, FormatterProfile,
    GlobalTypeDeclaration, GlobalTypesConfig, HtmlButtonHasTypeOptions, HtmlQuoteStyle,
    HtmlQuotesConfig, HtmlQuotesOptions, HtmlSelfClosingHtmlOptions, HtmlSelfClosingOptions,
    HtmlSelfClosingStyle, HyphenationStyle, ImportSortCustomGroup, ImportSortGroup, JsxCompat,
    JsxMode, LabelHasForOptions, LanguageServerConfig, LanguageServerUnstableFlags, LibConfig,
    LibRegistryConfig, LintRuleOptions, LintRuleSeverity, LinterConfig, LinterConfigEntry,
    LinterConfigPlan, LinterConfigPlanWithConfigRuleOptions, LinterConfigPlanWithRuleOptions,
    LinterExecutionOptions, LinterFeatureFlags, LspConfig, MaxLinesOptions,
    MaxLinesPerBlockOptions, MaxPropsOptions, MaxTemplateDepthOptions,
    MultiWordComponentNamesOptions, MuseaDesignToken, MuseaPreferDesignTokensOptions,
    MustacheInterpolationSpacingStyle, NoBareStringsInTemplateOptions,
    NoConfusingVoidExpressionOptions, NoMutatingPropsOptions, NoRestrictedGlobalsOptions,
    NoRestrictedMembersOptions, NoRestrictedSyntaxOptions, NoUnnecessaryConditionOptions,
    NoUseVIfWithVForOptions, ParseVueVersionError, PropNameCasingOptions, PropNameCasingStyle,
    PropsDestructureMode, QuoteProps, QuotePropsStyle, ResolvedLinterConfig,
    ResolvedLinterConfigWithConfigRuleOptions, RestrictPlusOperandsOptions,
    RestrictTemplateExpressionsOptions, RestrictedGlobal, RestrictedMember, RestrictedNames,
    RestrictedSyntaxSelector, ReturnAwaitOption, SfcElementOrderGroup, SfcElementOrderOptions,
    SlotNameCasingStyle, SortImportsConfig, SortImportsSetting, SortKeysOptions, SortKeysOrder,
    StrictBooleanExpressionsOptions, StyleAttributeMode, TemplateComponentNameCasing,
    TrailingComma, TypeAwareRuleOptions, TypeCheckerConfig, TypeSpecifier, TypeSpecifierNames,
    TypeSpecifierSource, UnsupportedFeaturesOptions, VBindStyle, VOnEventHyphenationConfig,
    VOnEventHyphenationOptions, VOnStyle, VSlotNamedStyle, VSlotStyle, VSlotStyleOptions,
    VSlotStyleValue, VSlotUniformStyle, VizeConfig, VueVersion, WarnCustomDirectiveOptions,
    esquery,
};
pub use normalize::normalize_public_config_value;
