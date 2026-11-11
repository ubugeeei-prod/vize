//! type/no-unnecessary-boolean-literal-compare
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};

static META: RuleMeta = RuleMeta {
    name: "type/no-unnecessary-boolean-literal-compare",
    description: "Disallow unnecessary comparisons against boolean literals.",
    category: RuleCategory::TypeAware,
    fixable: false,
    default_severity: Severity::Warning,
};

/// Metadata for `type/no-unnecessary-boolean-literal-compare`.
#[derive(Default)]
pub struct NoUnnecessaryBooleanLiteralCompare;

impl Rule for NoUnnecessaryBooleanLiteralCompare {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }
}
