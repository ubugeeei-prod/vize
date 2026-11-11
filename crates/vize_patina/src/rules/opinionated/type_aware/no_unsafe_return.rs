//! type/no-unsafe-return
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};

static META: RuleMeta = RuleMeta {
    name: "type/no-unsafe-return",
    description: "Disallow returning any-typed values from functions.",
    category: RuleCategory::TypeAware,
    fixable: false,
    default_severity: Severity::Warning,
};

/// Metadata for `type/no-unsafe-return`.
#[derive(Default)]
pub struct NoUnsafeReturn;

impl Rule for NoUnsafeReturn {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }
}
