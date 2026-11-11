//! type/no-array-delete
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};

static META: RuleMeta = RuleMeta {
    name: "type/no-array-delete",
    description: "Disallow deleting elements from array-like values.",
    category: RuleCategory::TypeAware,
    fixable: false,
    default_severity: Severity::Warning,
};

/// Metadata for `type/no-array-delete`.
#[derive(Default)]
pub struct NoArrayDelete;

impl Rule for NoArrayDelete {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }
}
