use std::sync::OnceLock;

use crate::fast::FastMap;

use super::{
    AwaitThenableRule, ConsistentReturnRule, ConsistentTypeExportsRule, DotNotationRule,
    LintDiagnostic, LintNode, NoArrayDeleteRule, NoBaseToStringRule, NoConfusingVoidExpressionRule,
    NoDeprecatedRule, NoDuplicateTypeConstituentsRule, NoFloatingPromisesRule, NoForInArrayRule,
    NoImpliedEvalRule, NoMeaninglessVoidOperatorRule, NoMisusedPromisesRule, NoMisusedSpreadRule,
    NoMixedEnumsRule, NoRedundantTypeConstituentsRule, NoUnnecessaryBooleanLiteralCompareRule,
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
    RestrictTemplateExpressionsRule, ReturnAwaitRule, RuleContext, RuleMeta, RustLintRule,
    StrictBooleanExpressionsRule, StrictVoidReturnRule, SwitchExhaustivenessCheckRule,
    UnboundMethodRule, UseUnknownInCatchCallbackVariableRule, host_facts,
};

/// Collection of Rust-authored lint rules addressable by stable rule name.
#[derive(Default)]
pub struct LintRuleRegistry {
    rules: Vec<Box<dyn RustLintRule>>,
    index: FastMap<&'static str, usize>,
}

impl LintRuleRegistry {
    /// Creates an empty registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds one rule to the registry and returns the updated registry.
    ///
    /// A rule registered under an already-present name replaces the earlier
    /// entry for lookup purposes.
    pub fn with_rule(mut self, rule: impl RustLintRule + 'static) -> Self {
        self.index.insert(rule.name(), self.rules.len());
        self.rules.push(Box::new(rule));
        self
    }

    /// Creates a registry containing the built-in type-aware lint rules.
    pub fn with_default_type_aware_rules() -> Self {
        Self::new()
            .with_rule(ConsistentReturnRule)
            .with_rule(ConsistentTypeExportsRule)
            .with_rule(DotNotationRule)
            .with_rule(NoArrayDeleteRule)
            .with_rule(NoBaseToStringRule)
            .with_rule(NoConfusingVoidExpressionRule)
            .with_rule(NoDeprecatedRule)
            .with_rule(NoDuplicateTypeConstituentsRule)
            .with_rule(NoFloatingPromisesRule)
            .with_rule(NoForInArrayRule)
            .with_rule(AwaitThenableRule)
            .with_rule(NoImpliedEvalRule)
            .with_rule(NoMeaninglessVoidOperatorRule)
            .with_rule(NoMisusedPromisesRule)
            .with_rule(NoMisusedSpreadRule)
            .with_rule(NoMixedEnumsRule)
            .with_rule(NoRedundantTypeConstituentsRule)
            .with_rule(NoUnnecessaryBooleanLiteralCompareRule)
            .with_rule(NoUnnecessaryConditionRule)
            .with_rule(NoUnnecessaryQualifierRule)
            .with_rule(NoUnnecessaryTemplateExpressionRule)
            .with_rule(NoUnnecessaryTypeArgumentsRule)
            .with_rule(NoUnnecessaryTypeAssertionRule)
            .with_rule(NoUnnecessaryTypeConversionRule)
            .with_rule(NoUnnecessaryTypeParametersRule)
            .with_rule(NoUnsafeArgumentRule)
            .with_rule(NoUnsafeAssignmentRule)
            .with_rule(NoUnsafeCallRule)
            .with_rule(NoUnsafeEnumComparisonRule)
            .with_rule(NoUnsafeMemberAccessRule)
            .with_rule(NoUnsafeReturnRule)
            .with_rule(NoUnsafeTypeAssertionRule)
            .with_rule(NoUnsafeUnaryMinusRule)
            .with_rule(NoUselessDefaultAssignmentRule)
            .with_rule(NonNullableTypeAssertionStyleRule)
            .with_rule(OnlyThrowErrorRule)
            .with_rule(PreferFindRule)
            .with_rule(PreferIncludesRule)
            .with_rule(PreferNullishCoalescingRule)
            .with_rule(PreferOptionalChainRule)
            .with_rule(PreferPromiseRejectErrorsRule)
            .with_rule(PreferReadonlyRule)
            .with_rule(PreferReadonlyParameterTypesRule)
            .with_rule(PreferReduceTypeParameterRule)
            .with_rule(PreferRegexpExecRule)
            .with_rule(PreferReturnThisTypeRule)
            .with_rule(PreferStringStartsEndsWithRule)
            .with_rule(PromiseFunctionAsyncRule)
            .with_rule(RelatedGetterSetterPairsRule)
            .with_rule(RequireArraySortCompareRule)
            .with_rule(RequireAwaitRule)
            .with_rule(RestrictPlusOperandsRule)
            .with_rule(RestrictTemplateExpressionsRule)
            .with_rule(ReturnAwaitRule)
            .with_rule(StrictBooleanExpressionsRule)
            .with_rule(StrictVoidReturnRule)
            .with_rule(SwitchExhaustivenessCheckRule)
            .with_rule(UnboundMethodRule)
            .with_rule(UseUnknownInCatchCallbackVariableRule)
    }

    /// Returns the stable names of every rule registered in insertion order.
    pub fn rule_names(&self) -> impl Iterator<Item = &'static str> + '_ {
        self.rules.iter().map(|rule| rule.name())
    }

    /// Returns serializable metadata for every registered rule.
    pub fn metas(&self) -> Vec<RuleMeta> {
        self.rules.iter().map(|rule| rule.meta()).collect()
    }

    /// Runs a single rule by name against one compact host-provided node.
    ///
    /// Returns `None` when `rule_name` is not registered.
    pub fn run_rule(&self, rule_name: &str, node: &LintNode) -> Option<Vec<LintDiagnostic>> {
        self.run_rule_owned(rule_name, node.clone())
    }

    /// Runs a single rule by name, taking ownership of the node.
    ///
    /// Hosts that deserialize a fresh [`LintNode`] per call (the FFI and
    /// JavaScript bridges) should prefer this over [`run_rule`](Self::run_rule)
    /// so derived host facts are applied in place instead of onto a deep copy.
    ///
    /// Returns `None` when `rule_name` is not registered.
    pub fn run_rule_owned(&self, rule_name: &str, node: LintNode) -> Option<Vec<LintDiagnostic>> {
        let rule = self.index.get(rule_name).map(|&slot| &self.rules[slot])?;
        let mut ctx = RuleContext::new(rule.as_ref());
        let prepared = host_facts::prepare_node_for_rule_owned(node);
        rule.check(&mut ctx, &prepared);
        Some(ctx.finish())
    }
}

/// Returns the process-wide registry of built-in type-aware lint rules.
///
/// The registry is built once and reused, so per-node bridge calls avoid
/// re-allocating all rules and re-scanning rule names.
pub fn default_type_aware_registry() -> &'static LintRuleRegistry {
    static REGISTRY: OnceLock<LintRuleRegistry> = OnceLock::new();
    REGISTRY.get_or_init(LintRuleRegistry::with_default_type_aware_rules)
}

/// Runs one built-in type-aware rule by name against a host-provided node.
///
/// This convenience helper resolves the rule through the shared
/// [`default_type_aware_registry`] and is safe to call once per visited node.
pub fn run_default_type_aware_rule(
    rule_name: &str,
    node: &LintNode,
) -> Option<Vec<LintDiagnostic>> {
    default_type_aware_registry().run_rule(rule_name, node)
}

/// Owned-node variant of [`run_default_type_aware_rule`].
///
/// Prefer this from FFI hosts that already own a freshly deserialized node.
pub fn run_default_type_aware_rule_owned(
    rule_name: &str,
    node: LintNode,
) -> Option<Vec<LintDiagnostic>> {
    default_type_aware_registry().run_rule_owned(rule_name, node)
}
