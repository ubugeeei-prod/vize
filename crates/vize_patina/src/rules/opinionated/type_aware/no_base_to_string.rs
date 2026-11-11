//! type/no-base-to-string
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};

static META: RuleMeta = RuleMeta {
    name: "type/no-base-to-string",
    description: "Disallow stringifying values that fall back to Object.prototype.toString().",
    category: RuleCategory::TypeAware,
    fixable: false,
    default_severity: Severity::Warning,
};

/// Metadata for `type/no-base-to-string`.
#[derive(Default)]
pub struct NoBaseToString;

impl Rule for NoBaseToString {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }
}
