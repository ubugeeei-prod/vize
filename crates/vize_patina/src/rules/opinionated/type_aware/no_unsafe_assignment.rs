//! type/no-unsafe-assignment
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};

static META: RuleMeta = RuleMeta {
    name: "type/no-unsafe-assignment",
    description: "Disallow assigning any-typed values to more specific targets.",
    category: RuleCategory::TypeAware,
    fixable: false,
    default_severity: Severity::Warning,
};

/// Metadata for `type/no-unsafe-assignment`.
#[derive(Default)]
pub struct NoUnsafeAssignment;

impl Rule for NoUnsafeAssignment {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }
}
