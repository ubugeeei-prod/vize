//! type/await-thenable
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};

static META: RuleMeta = RuleMeta {
    name: "type/await-thenable",
    description: "Disallow awaiting non-thenable values.",
    category: RuleCategory::TypeAware,
    fixable: false,
    default_severity: Severity::Warning,
};

/// Metadata for `type/await-thenable`.
#[derive(Default)]
pub struct AwaitThenable;

impl Rule for AwaitThenable {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }
}
