//! type/no-meaningless-void-operator
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};

static META: RuleMeta = RuleMeta {
    name: "type/no-meaningless-void-operator",
    description: "Disallow void operators on expressions that are already void-like.",
    category: RuleCategory::TypeAware,
    fixable: false,
    default_severity: Severity::Warning,
};

/// Metadata for `type/no-meaningless-void-operator`.
#[derive(Default)]
pub struct NoMeaninglessVoidOperator;

impl Rule for NoMeaninglessVoidOperator {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }
}
