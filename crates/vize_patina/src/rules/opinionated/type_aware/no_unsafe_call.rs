//! type/no-unsafe-call
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};

static META: RuleMeta = RuleMeta {
    name: "type/no-unsafe-call",
    description: "Disallow calling values typed as any.",
    category: RuleCategory::TypeAware,
    fixable: false,
    default_severity: Severity::Warning,
};

/// Metadata for `type/no-unsafe-call`.
#[derive(Default)]
pub struct NoUnsafeCall;

impl Rule for NoUnsafeCall {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }
}
