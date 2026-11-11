//! type/no-unsafe-enum-assignment
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};

static META: RuleMeta = RuleMeta {
    name: "type/no-unsafe-enum-assignment",
    description: "Disallow assigning a non-enum value to an enum.",
    category: RuleCategory::TypeAware,
    fixable: false,
    default_severity: Severity::Warning,
};

/// Metadata for `type/no-unsafe-enum-assignment`.
#[derive(Default)]
pub struct NoUnsafeEnumAssignment;

impl Rule for NoUnsafeEnumAssignment {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }
}
