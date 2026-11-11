//! type/no-misused-promises
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};

static META: RuleMeta = RuleMeta {
    name: "type/no-misused-promises",
    description: "Disallow Promises in places not designed to handle them.",
    category: RuleCategory::TypeAware,
    fixable: false,
    default_severity: Severity::Warning,
};

/// Metadata for `type/no-misused-promises`.
#[derive(Default)]
pub struct NoMisusedPromises;

impl Rule for NoMisusedPromises {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }
}
