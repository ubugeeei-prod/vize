//! type/require-await
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};

static META: RuleMeta = RuleMeta {
    name: "type/require-await",
    description: "Disallow async functions that contain no await expression.",
    category: RuleCategory::TypeAware,
    fixable: false,
    default_severity: Severity::Warning,
};

/// Metadata for `type/require-await`.
#[derive(Default)]
pub struct RequireAwait;

impl Rule for RequireAwait {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }
}
