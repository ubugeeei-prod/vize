//! In-memory Vize configuration values and projections.

mod document;
mod model;
mod normalize;
pub use crate::dialect::VueDialect;
pub use document::ConfigDocument;
pub use model::{
    AllowListOptions, AnchorHasContentOptions, ArrowParens, AttributeHyphenationConfig,
    AttributeHyphenationOptions, AttributeSortOrder, AwaitThenableOptions, BlockLangBlock,
    BlockLangNames, BlockLangOptions, CallAfterAwaitOptions, CallAfterAwaitSpec,
    ChecksConditionalsOptions, ChecksVoidReturnOptions, CompilerConfigEntry,
    ComponentNameInTemplateCasingOptions, ConfigEntryFiles, ConfigEntryIgnore,
    ConfigExperimentalVueFlags, ConfigFeatureFlags, ConfigLintRuleOptions,
    ConfiguredVueRuleOptions, ConstantLoopConditionMode, ConstantLoopConditions,
    CustomEventNameCasing, CustomEventNameCasingOptions, DefineMacrosOrderOptions,
    DefinePropsDestructuringOptions, EndOfLine, EnforceStyleAttributeOptions, FormatterConfig,
    FormatterPreset, FormatterProfile, GlobalTypeDeclaration, GlobalTypesConfig,
    HeadingHasContentOptions, HtmlButtonHasTypeOptions, HtmlQuoteStyle, HtmlQuotesConfig,
    HtmlQuotesOptions, HtmlSelfClosingHtmlOptions, HtmlSelfClosingOptions, HtmlSelfClosingStyle,
    HyphenationStyle, ImportSortCustomGroup, ImportSortGroup, JsxCompat, JsxMode,
    LabelHasForOptions, LanguageServerConfig, LanguageServerUnstableFlags, LibConfig,
    LibRegistryConfig, LintRuleOptions, LintRuleSeverity, LinterConfig, LinterConfigEntry,
    LinterConfigPlan, LinterConfigPlanWithConfigRuleOptions, LinterConfigPlanWithRuleOptions,
    LinterExecutionOptions, LinterFeatureFlags, LspConfig, MaxLinesOptions,
    MaxLinesPerBlockOptions, MaxPropsOptions, MaxTemplateDepthOptions,
    MultiWordComponentNamesOptions, MuseaDesignToken, MuseaPreferDesignTokensOptions,
    MustacheInterpolationSpacingStyle, NoBareStringsInTemplateOptions, NoBaseToStringOptions,
    NoConfusingVoidExpressionOptions, NoDuplicateTypeConstituentsOptions,
    NoFloatingPromisesOptions, NoMeaninglessVoidOperatorOptions, NoMisusedPromisesOptions,
    NoMutatingPropsOptions, NoRestrictedGlobalsOptions, NoRestrictedMembersOptions,
    NoRestrictedSyntaxOptions, NoUnnecessaryBooleanLiteralCompareOptions,
    NoUnnecessaryConditionOptions, NoUnnecessaryTypeAssertionOptions, NoUnsafeMemberAccessOptions,
    NoUnusedScriptVarsOptions, NoUseVIfWithVForOptions, OnlyThrowErrorOptions,
    ParseVueVersionError, PreferConstDestructuring, PreferConstOptions,
    PreferPromiseRejectErrorsOptions, PromiseUnionFlag, PropNameCasingOptions, PropNameCasingStyle,
    PropsDestructureMode, QuoteProps, QuotePropsStyle, ResolvedLinterConfig,
    ResolvedLinterConfigWithConfigRuleOptions, RestrictPlusOperandsOptions,
    RestrictTemplateExpressionsOptions, RestrictedGlobal, RestrictedMember, RestrictedNames,
    RestrictedSyntaxSelector, ReturnAwaitOption, SfcElementOrderGroup, SfcElementOrderOptions,
    SlotNameCasingStyle, SortImportsConfig, SortImportsSetting, SortKeysOptions, SortKeysOrder,
    StrictBooleanExpressionsOptions, StyleAttributeMode, Switch, TemplateComponentNameCasing,
    TrailingComma, TypeAwareRuleOptions, TypeCheckerConfig, TypeSpecifier, TypeSpecifierNames,
    TypeSpecifierSource, TypeSpecifiers, UnboundMethodOptions, UnsupportedFeaturesOptions,
    UnusedArgs, UnusedCaughtErrors, UnusedVarsScope, VBindStyle, VOnEventHyphenationConfig,
    VOnEventHyphenationOptions, VOnHandlerAllowed, VOnHandlerKind, VOnHandlerStyleConfig,
    VOnHandlerStyleOptions, VOnStyle, VSlotNamedStyle, VSlotStyle, VSlotStyleOptions,
    VSlotStyleValue, VSlotUniformStyle, VizeConfig, VueVersion, WarnCustomDirectiveOptions,
    esquery,
};
pub use normalize::normalize_public_config_value;
