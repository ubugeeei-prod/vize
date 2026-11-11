//! type/no-unnecessary-condition
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};

static META: RuleMeta = RuleMeta {
    name: "type/no-unnecessary-condition",
    description: "Disallow conditions that are always truthy, always falsy, never, or always nullish.",
    category: RuleCategory::TypeAware,
    fixable: false,
    default_severity: Severity::Warning,
};

/// Metadata for `type/no-unnecessary-condition`.
#[derive(Default)]
pub struct NoUnnecessaryCondition;

impl Rule for NoUnnecessaryCondition {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }
}
