//! type/no-unsafe-argument
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};

static META: RuleMeta = RuleMeta {
    name: "type/no-unsafe-argument",
    description: "Disallow calling a function with a value of type `any`.",
    category: RuleCategory::TypeAware,
    fixable: false,
    default_severity: Severity::Warning,
};

/// Metadata for `type/no-unsafe-argument`.
#[derive(Default)]
pub struct NoUnsafeArgument;

impl Rule for NoUnsafeArgument {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }
}
