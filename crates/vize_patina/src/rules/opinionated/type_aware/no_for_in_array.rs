//! type/no-for-in-array
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};

static META: RuleMeta = RuleMeta {
    name: "type/no-for-in-array",
    description: "Disallow for-in iteration over array-like values.",
    category: RuleCategory::TypeAware,
    fixable: false,
    default_severity: Severity::Warning,
};

/// Metadata for `type/no-for-in-array`.
#[derive(Default)]
pub struct NoForInArray;

impl Rule for NoForInArray {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }
}
