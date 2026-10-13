//! Rust-authored lint rule primitives and built-in type-aware rules.

mod context;
mod helpers;
mod host_facts;
mod registry;
mod rules;
#[cfg(test)]
mod tests;
mod types;

pub use context::{RuleContext, RustLintRule};
pub use registry::{
    LintRuleRegistry, default_type_aware_registry, run_default_type_aware_rule,
    run_default_type_aware_rule_owned,
};
pub use rules::{
    AwaitThenableRule, ConsistentReturnRule, ConsistentTypeExportsRule, DotNotationRule,
    NoArrayDeleteRule, NoBaseToStringRule, NoConfusingVoidExpressionRule, NoDeprecatedRule,
    NoDuplicateTypeConstituentsRule, NoFloatingPromisesRule, NoForInArrayRule, NoImpliedEvalRule,
    NoMeaninglessVoidOperatorRule, NoMisusedPromisesRule, NoMisusedSpreadRule, NoMixedEnumsRule,
    NoRedundantTypeConstituentsRule, NoUnnecessaryBooleanLiteralCompareRule,
    NoUnnecessaryConditionRule, NoUnnecessaryQualifierRule, NoUnnecessaryTemplateExpressionRule,
    NoUnnecessaryTypeArgumentsRule, NoUnnecessaryTypeAssertionRule,
    NoUnnecessaryTypeConversionRule, NoUnnecessaryTypeParametersRule, NoUnsafeArgumentRule,
    NoUnsafeAssignmentRule, NoUnsafeCallRule, NoUnsafeEnumComparisonRule, NoUnsafeMemberAccessRule,
    NoUnsafeReturnRule, NoUnsafeTypeAssertionRule, NoUnsafeUnaryMinusRule,
    NoUselessDefaultAssignmentRule, NonNullableTypeAssertionStyleRule, OnlyThrowErrorRule,
    PreferFindRule, PreferIncludesRule, PreferNullishCoalescingRule, PreferOptionalChainRule,
    PreferPromiseRejectErrorsRule, PreferReadonlyParameterTypesRule, PreferReadonlyRule,
    PreferReduceTypeParameterRule, PreferRegexpExecRule, PreferReturnThisTypeRule,
    PreferStringStartsEndsWithRule, PromiseFunctionAsyncRule, RelatedGetterSetterPairsRule,
    RequireArraySortCompareRule, RequireAwaitRule, RestrictPlusOperandsRule,
    RestrictTemplateExpressionsRule, ReturnAwaitRule, StrictBooleanExpressionsRule,
    StrictVoidReturnRule, SwitchExhaustivenessCheckRule, UnboundMethodRule,
    UseUnknownInCatchCallbackVariableRule,
};
pub use types::{
    LintDiagnostic, LintFix, LintNode, LintSuggestion, NodeMetadataDepth, RuleBridgeRequirements,
    RuleMessage, RuleMeta, TextRange,
};
