//! type/no-confusing-void-expression
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};

static META: RuleMeta = RuleMeta {
    name: "type/no-confusing-void-expression",
    description: "Disallow void-typed expressions in confusing value positions.",
    category: RuleCategory::TypeAware,
    fixable: false,
    default_severity: Severity::Warning,
};

/// Metadata for `type/no-confusing-void-expression`.
#[derive(Default)]
pub struct NoConfusingVoidExpression;

impl Rule for NoConfusingVoidExpression {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }
}
