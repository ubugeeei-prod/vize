//! In-memory Vize configuration values and projections.

mod directive;
mod document;
mod model;
mod normalize;
pub use crate::dialect::VueDialect;
pub use directive::{
    DirectiveContract, DirectiveContractError, DirectiveContracts, DirectiveModifier,
    NO_DIRECTIVE_CONTRACTS,
};
pub use document::ConfigDocument;
pub use model::{
    AllowListOptions, AnchorHasContentOptions, ArrowParens, AttributeHyphenationConfig,
    AttributeHyphenationOptions, AttributeSortOrder, AwaitThenableOptions, BlockLangBlock,
    BlockLangNames, BlockLangOptions, BlockSpacing, CallAfterAwaitOptions, CallAfterAwaitSpec,
    ChecksConditionalsOptions, ChecksVoidReturnOptions, CompilerConfigEntry,
    ComponentDefinitionNameCasingStyle, ComponentNameInTemplateCasingOptions, ConfigEntryFiles,
    ConfigEntryIgnore, ConfigExperimentalVueFlags, ConfigFeatureFlags, ConfigLintRuleOptions,
    ConfiguredVueRuleOptions, ConsistentTypeImportsOptions, ConstantLoopConditionMode,
    ConstantLoopConditions, CustomEventNameCasing, CustomEventNameCasingOptions,
    DefineMacrosOrderOptions, DefinePropsDestructuringOptions, EndOfLine,
    EnforceStyleAttributeOptions, FormatterConfig, FormatterPreset, FormatterProfile,
    GlobalTypeDeclaration, GlobalTypesConfig, HeadingHasContentOptions, HtmlButtonHasTypeOptions,
    HtmlQuoteStyle, HtmlQuotesConfig, HtmlQuotesOptions, HtmlSelfClosingHtmlOptions,
    HtmlSelfClosingOptions, HtmlSelfClosingStyle, HyphenationStyle, ImportSortCustomGroup,
    ImportSortGroup, JsxCompat, JsxMode, LabelHasForOptions, LanguageServerConfig,
    LanguageServerUnstableFlags, LibConfig, LibRegistryConfig, LintRuleOptions, LintRuleSeverity,
    LinterConfig, LinterConfigEntry, LinterConfigPlan, LinterConfigPlanWithConfigRuleOptions,
    LinterConfigPlanWithRuleOptions, LinterExecutionOptions, LinterFeatureFlags, LspConfig,
    MaxLinesOptions, MaxLinesPerBlockOptions, MaxPropsOptions, MaxTemplateDepthOptions,
    MultiWordComponentNamesOptions, MuseaDesignToken, MuseaPreferDesignTokensOptions,
    MustacheInterpolationSpacingStyle, NoBareStringsInTemplateOptions, NoBaseToStringOptions,
    NoConfusingVoidExpressionOptions, NoDuplicateTypeConstituentsOptions,
    NoFloatingPromisesOptions, NoMeaninglessVoidOperatorOptions, NoMisusedPromisesOptions,
    NoMutatingPropsOptions, NoRestrictedGlobalsOptions, NoRestrictedMembersOptions,
    NoRestrictedSyntaxOptions, NoUnnecessaryBooleanLiteralCompareOptions,
    NoUnnecessaryConditionOptions, NoUnnecessaryTypeAssertionOptions, NoUnsafeMemberAccessOptions,
    NoUnusedScriptVarsOptions, NoUseVIfWithVForOptions, ObjectWrap, OnlyThrowErrorOptions,
    ParseVueVersionError, PreferConstDestructuring, PreferConstOptions,
    PreferPromiseRejectErrorsOptions, PromiseUnionFlag, PropNameCasingOptions, PropNameCasingStyle,
    PropsDestructureMode, QuoteProps, QuotePropsStyle, ResolvedLinterConfig,
    ResolvedLinterConfigWithConfigRuleOptions, RestrictPlusOperandsOptions,
    RestrictTemplateExpressionsOptions, RestrictedGlobal, RestrictedHtmlElement,
    RestrictedHtmlElementSpec, RestrictedHtmlElements, RestrictedMember, RestrictedNames,
    RestrictedSyntaxSelector, RestrictedVOn, RestrictedVOnEntry, RestrictedVOnSpec,
    ReturnAwaitOption, SfcElementOrderGroup, SfcElementOrderOptions, SlotNameCasingStyle,
    SortImportsConfig, SortImportsSetting, SortKeysOptions, SortKeysOrder,
    StrictBooleanExpressionsOptions, StyleAttributeMode, Switch, TemplateComponentNameCasing,
    TrailingComma, TypeAwareRuleOptions, TypeCheckerConfig, TypeImportFixStyle,
    TypeImportPreference, TypeSpecifier, TypeSpecifierNames, TypeSpecifierSource, TypeSpecifiers,
    UnboundMethodOptions, UnsupportedFeaturesOptions, UnusedArgs, UnusedCaughtErrors,
    UnusedVarsScope, VBindSameNameShorthand, VBindStyle, VBindStyleConfig, VBindStyleOptions,
    VOnEventHyphenationConfig, VOnEventHyphenationOptions, VOnHandlerAllowed, VOnHandlerKind,
    VOnHandlerStyleConfig, VOnHandlerStyleOptions, VOnStyle, VSlotNamedStyle, VSlotStyle,
    VSlotStyleOptions, VSlotStyleValue, VSlotUniformStyle, VizeConfig, VueVersion,
    WarnCustomDirectiveOptions, esquery,
};
pub use normalize::normalize_public_config_value;
