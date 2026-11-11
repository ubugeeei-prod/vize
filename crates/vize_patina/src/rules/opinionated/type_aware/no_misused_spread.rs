//! type/no-misused-spread
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};

static META: RuleMeta = RuleMeta {
    name: "type/no-misused-spread",
    description: "Disallow spreading values into incompatible positions.",
    category: RuleCategory::TypeAware,
    fixable: false,
    default_severity: Severity::Warning,
};

/// Metadata for `type/no-misused-spread`.
#[derive(Default)]
pub struct NoMisusedSpread;

impl Rule for NoMisusedSpread {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }
}
