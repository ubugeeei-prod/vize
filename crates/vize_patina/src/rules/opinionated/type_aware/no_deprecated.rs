//! type/no-deprecated
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};

static META: RuleMeta = RuleMeta {
    name: "type/no-deprecated",
    description: "Disallow use of deprecated declarations.",
    category: RuleCategory::TypeAware,
    fixable: false,
    default_severity: Severity::Warning,
};

/// Metadata for `type/no-deprecated`.
#[derive(Default)]
pub struct NoDeprecated;

impl Rule for NoDeprecated {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }
}
