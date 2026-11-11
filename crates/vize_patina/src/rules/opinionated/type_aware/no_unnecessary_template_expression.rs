//! type/no-unnecessary-template-expression
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};

static META: RuleMeta = RuleMeta {
    name: "type/no-unnecessary-template-expression",
    description: "Disallow template literals that wrap a single expression unnecessarily.",
    category: RuleCategory::TypeAware,
    fixable: false,
    default_severity: Severity::Warning,
};

/// Metadata for `type/no-unnecessary-template-expression`.
#[derive(Default)]
pub struct NoUnnecessaryTemplateExpression;

impl Rule for NoUnnecessaryTemplateExpression {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }
}
